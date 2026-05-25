package sys

import (
	"auroraops/internal/service"
	"bufio"
	"bytes"
	"context"
	"encoding/binary"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"strings"
	"sync"
	"time"

	"github.com/gogf/gf/v2/frame/g"
	"github.com/gogf/gf/v2/net/ghttp"
	"github.com/gorilla/websocket"
)

const (
	weylusTunnelWriteWait  = 10 * time.Second
	weylusTunnelOpenWait   = 10 * time.Second
	weylusTunnelFrameOpen  = byte(1)
	weylusTunnelFrameData  = byte(2)
	weylusTunnelFrameClose = byte(3)
	weylusTunnelChunkSize  = 64 * 1024
)

type weylusTunnelOpen struct {
	StreamID uint64 `json:"streamId"`
}

type weylusTunnelStream struct {
	id     uint64
	data   chan []byte
	closed chan struct{}
	once   sync.Once
}

type weylusTunnel struct {
	deviceID uint64
	conn     *websocket.Conn
	send     chan []byte
	streams  map[uint64]*weylusTunnelStream
	mu       sync.Mutex
	nextID   uint64
	closed   chan struct{}
	once     sync.Once
}

var (
	weylusTunnelUpgrader = websocket.Upgrader{
		ReadBufferSize:  64 * 1024,
		WriteBufferSize: 64 * 1024,
		CheckOrigin: func(r *http.Request) bool {
			return true
		},
	}
	weylusTunnels = struct {
		sync.RWMutex
		byDevice map[uint64]*weylusTunnel
	}{byDevice: make(map[uint64]*weylusTunnel)}
)

func (c *cOpsDevice) WeylusTunnelWS(r *ghttp.Request) {
	deviceID := r.Get("deviceId").Uint64()
	hostname := r.Get("hostname").String()
	token := r.Get("token").String()
	if deviceID == 0 || hostname == "" || token == "" {
		r.Response.WriteStatusExit(http.StatusBadRequest, "deviceId, hostname and token are required")
		return
	}
	if err := service.SysOpsDevice().VerifyClientToken(r.Context(), deviceID, hostname, token); err != nil {
		r.Response.WriteStatusExit(http.StatusUnauthorized, err.Error())
		return
	}

	conn, err := weylusTunnelUpgrader.Upgrade(r.Response.Writer, r.Request, nil)
	if err != nil {
		return
	}
	tunnel := newWeylusTunnel(deviceID, conn)
	registerWeylusTunnel(tunnel)
	tunnel.run(r.Context())
}

func newWeylusTunnel(deviceID uint64, conn *websocket.Conn) *weylusTunnel {
	return &weylusTunnel{
		deviceID: deviceID,
		conn:     conn,
		send:     make(chan []byte, 256),
		streams:  make(map[uint64]*weylusTunnelStream),
		closed:   make(chan struct{}),
	}
}

func registerWeylusTunnel(tunnel *weylusTunnel) {
	weylusTunnels.Lock()
	if old := weylusTunnels.byDevice[tunnel.deviceID]; old != nil {
		old.close()
	}
	weylusTunnels.byDevice[tunnel.deviceID] = tunnel
	weylusTunnels.Unlock()
	g.Log().Infof(context.Background(), "weylus tunnel registered: deviceId=%d", tunnel.deviceID)
}

func getWeylusTunnel(deviceID uint64) *weylusTunnel {
	weylusTunnels.RLock()
	tunnel := weylusTunnels.byDevice[deviceID]
	weylusTunnels.RUnlock()
	return tunnel
}

func unregisterWeylusTunnel(tunnel *weylusTunnel) {
	weylusTunnels.Lock()
	if weylusTunnels.byDevice[tunnel.deviceID] == tunnel {
		delete(weylusTunnels.byDevice, tunnel.deviceID)
		g.Log().Infof(context.Background(), "weylus tunnel unregistered: deviceId=%d", tunnel.deviceID)
	}
	weylusTunnels.Unlock()
}

func (t *weylusTunnel) run(ctx context.Context) {
	defer unregisterWeylusTunnel(t)
	defer t.close()

	go t.writeLoop()
	for {
		messageType, payload, err := t.conn.ReadMessage()
		if err != nil {
			return
		}
		if messageType != websocket.BinaryMessage || len(payload) < 9 {
			continue
		}
		frameType := payload[0]
		streamID := binary.BigEndian.Uint64(payload[1:9])
		body := payload[9:]
		switch frameType {
		case weylusTunnelFrameData:
			t.dispatchData(streamID, body)
		case weylusTunnelFrameClose:
			t.closeStream(streamID)
		}

		select {
		case <-ctx.Done():
			return
		default:
		}
	}
}

func (t *weylusTunnel) writeLoop() {
	for {
		select {
		case payload := <-t.send:
			_ = t.conn.SetWriteDeadline(time.Now().Add(weylusTunnelWriteWait))
			if err := t.conn.WriteMessage(websocket.BinaryMessage, payload); err != nil {
				t.close()
				return
			}
		case <-t.closed:
			return
		}
	}
}

func (t *weylusTunnel) openStream() (*weylusTunnelStream, error) {
	t.mu.Lock()
	select {
	case <-t.closed:
		t.mu.Unlock()
		return nil, errors.New("weylus tunnel closed")
	default:
	}
	t.nextID++
	stream := &weylusTunnelStream{
		id:     t.nextID,
		data:   make(chan []byte, 128),
		closed: make(chan struct{}),
	}
	t.streams[stream.id] = stream
	t.mu.Unlock()

	body, _ := json.Marshal(weylusTunnelOpen{StreamID: stream.id})
	if !t.sendFrame(weylusTunnelFrameOpen, stream.id, body) {
		t.closeStream(stream.id)
		return nil, errors.New("weylus tunnel send failed")
	}
	return stream, nil
}

func (t *weylusTunnel) dispatchData(streamID uint64, data []byte) {
	t.mu.Lock()
	stream := t.streams[streamID]
	t.mu.Unlock()
	if stream == nil {
		return
	}
	cp := append([]byte(nil), data...)
	select {
	case stream.data <- cp:
	case <-stream.closed:
	}
}

func (t *weylusTunnel) closeStream(streamID uint64) {
	t.mu.Lock()
	stream := t.streams[streamID]
	if stream != nil {
		delete(t.streams, streamID)
	}
	t.mu.Unlock()
	if stream != nil {
		stream.once.Do(func() {
			close(stream.closed)
			close(stream.data)
		})
	}
}

func (t *weylusTunnel) sendData(streamID uint64, data []byte) bool {
	for len(data) > 0 {
		n := len(data)
		if n > weylusTunnelChunkSize {
			n = weylusTunnelChunkSize
		}
		if !t.sendFrame(weylusTunnelFrameData, streamID, data[:n]) {
			return false
		}
		data = data[n:]
	}
	return true
}

func (t *weylusTunnel) sendClose(streamID uint64) {
	_ = t.sendFrame(weylusTunnelFrameClose, streamID, nil)
	t.closeStream(streamID)
}

func (t *weylusTunnel) sendFrame(frameType byte, streamID uint64, body []byte) bool {
	frame := make([]byte, 9+len(body))
	frame[0] = frameType
	binary.BigEndian.PutUint64(frame[1:9], streamID)
	copy(frame[9:], body)
	select {
	case t.send <- frame:
		return true
	case <-t.closed:
		return false
	}
}

func (t *weylusTunnel) close() {
	t.once.Do(func() {
		close(t.closed)
		_ = t.conn.Close()
		t.mu.Lock()
		for id, stream := range t.streams {
			delete(t.streams, id)
			stream.once.Do(func() {
				close(stream.closed)
				close(stream.data)
			})
		}
		t.mu.Unlock()
	})
}

func proxyWeylusViaTunnel(r *ghttp.Request, tunnel *weylusTunnel) bool {
	g.Log().Debugf(r.Context(), "weylus proxy via tunnel: deviceId=%d path=%s", tunnel.deviceID, r.URL.Path)
	if strings.EqualFold(r.Header.Get("Upgrade"), "websocket") {
		proxyWeylusWebSocketViaTunnel(r, tunnel)
		return true
	}
	proxyWeylusHTTPViaTunnel(r, tunnel)
	return true
}

func proxyWeylusHTTPViaTunnel(r *ghttp.Request, tunnel *weylusTunnel) {
	stream, err := tunnel.openStream()
	if err != nil {
		r.Response.WriteStatusExit(http.StatusBadGateway, err.Error())
		return
	}
	defer tunnel.sendClose(stream.id)

	rawReq, err := buildWeylusRawRequest(r, false)
	if err != nil {
		r.Response.WriteStatusExit(http.StatusBadRequest, err.Error())
		return
	}
	if !tunnel.sendData(stream.id, rawReq) {
		r.Response.WriteStatusExit(http.StatusBadGateway, "weylus tunnel send failed")
		return
	}

	var rawResp bytes.Buffer
	timer := time.NewTimer(weylusTunnelOpenWait)
	defer timer.Stop()
	for {
		select {
		case data, ok := <-stream.data:
			if !ok {
				writeWeylusHTTPResponse(r, &rawResp)
				return
			}
			_, _ = rawResp.Write(data)
		case <-timer.C:
			r.Response.WriteStatusExit(http.StatusGatewayTimeout, "weylus tunnel response timeout")
			return
		case <-r.Context().Done():
			return
		}
	}
}

func proxyWeylusWebSocketViaTunnel(r *ghttp.Request, tunnel *weylusTunnel) {
	stream, err := tunnel.openStream()
	if err != nil {
		r.Response.WriteStatusExit(http.StatusBadGateway, err.Error())
		return
	}
	defer tunnel.sendClose(stream.id)

	rawReq, err := buildWeylusRawRequest(r, true)
	if err != nil {
		r.Response.WriteStatusExit(http.StatusBadRequest, err.Error())
		return
	}

	clientConn, clientBuf, err := r.Response.RawWriter().(http.Hijacker).Hijack()
	if err != nil {
		return
	}
	defer clientConn.Close()

	if !tunnel.sendData(stream.id, rawReq) {
		return
	}
	go pipeClientToWeylusTunnel(clientConn, clientBuf, tunnel, stream.id)
	for data := range stream.data {
		if _, err = clientConn.Write(data); err != nil {
			return
		}
	}
}

func pipeClientToWeylusTunnel(conn io.Reader, reader *bufio.ReadWriter, tunnel *weylusTunnel, streamID uint64) {
	defer tunnel.sendClose(streamID)
	if reader != nil && reader.Reader.Buffered() > 0 {
		buffered, _ := io.ReadAll(reader.Reader)
		if len(buffered) > 0 && !tunnel.sendData(streamID, buffered) {
			return
		}
	}
	buf := make([]byte, weylusTunnelChunkSize)
	for {
		n, err := conn.Read(buf)
		if n > 0 && !tunnel.sendData(streamID, buf[:n]) {
			return
		}
		if err != nil {
			return
		}
	}
}

func buildWeylusRawRequest(r *ghttp.Request, websocketUpgrade bool) ([]byte, error) {
	targetPath := strings.TrimPrefix(r.URL.Path, weylusProxyPrefix)
	if targetPath == "" {
		targetPath = "/"
	}
	if r.URL.RawQuery != "" {
		targetPath += "?" + r.URL.RawQuery
	}

	var b bytes.Buffer
	fmt.Fprintf(&b, "%s %s HTTP/1.1\r\n", r.Method, targetPath)
	b.WriteString("Host: 127.0.0.1:1701\r\n")
	for key, values := range r.Header {
		if strings.EqualFold(key, "Host") {
			continue
		}
		if !websocketUpgrade && strings.EqualFold(key, "Accept-Encoding") {
			continue
		}
		if !websocketUpgrade && strings.EqualFold(key, "Connection") {
			continue
		}
		for _, value := range values {
			fmt.Fprintf(&b, "%s: %s\r\n", key, value)
		}
	}
	if !websocketUpgrade {
		b.WriteString("Connection: close\r\n")
	}
	b.WriteString("\r\n")
	if r.Body != nil {
		body, err := io.ReadAll(r.Body)
		if err != nil {
			return nil, err
		}
		_, _ = b.Write(body)
	}
	return b.Bytes(), nil
}

func writeWeylusHTTPResponse(r *ghttp.Request, rawResp *bytes.Buffer) {
	resp, err := http.ReadResponse(bufio.NewReader(bytes.NewReader(rawResp.Bytes())), r.Request)
	if err != nil {
		r.Response.WriteStatusExit(http.StatusBadGateway, err.Error())
		return
	}
	defer resp.Body.Close()

	body, _ := io.ReadAll(resp.Body)
	body = rewriteWeylusBody(resp.Header.Get("Content-Type"), resp.Request.URL.Path, r.URL.RawQuery, body)
	for key, values := range resp.Header {
		if strings.EqualFold(key, "Content-Length") {
			continue
		}
		for _, value := range values {
			r.Response.Header().Add(key, value)
		}
	}
	r.Response.Status = resp.StatusCode
	r.Response.Write(body)
}

func rewriteWeylusBody(contentType, path, rawQuery string, body []byte) []byte {
	querySuffix := ""
	if rawQuery != "" {
		querySuffix = "?" + rawQuery
	}
	if strings.Contains(contentType, "text/html") {
		body = bytes.ReplaceAll(body, []byte(`href="style.css"`), []byte(fmt.Sprintf(`href="%s/style.css%s"`, weylusProxyPrefix, querySuffix)))
		body = bytes.ReplaceAll(body, []byte(`src="lib.js"`), []byte(fmt.Sprintf(`src="%s/lib.js%s"`, weylusProxyPrefix, querySuffix)))
		return rewriteWeylusBranding(body)
	}
	if strings.Contains(contentType, "javascript") || strings.HasSuffix(path, "/lib.js") {
		return bytes.ReplaceAll(body, []byte(`"/ws"`), []byte(fmt.Sprintf(`"%s/ws%s"`, weylusProxyPrefix, querySuffix)))
	}
	return body
}
