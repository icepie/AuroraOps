package sys

import (
	"auroraops/api/admin/opsdevice"
	"auroraops/internal/model/input/sysin"
	"auroraops/internal/service"
	"bytes"
	"context"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/http/httputil"
	"net/url"
	"strings"
	"time"

	"github.com/gogf/gf/v2/net/ghttp"
	"github.com/gorilla/websocket"
)

var (
	OpsDevice = cOpsDevice{}
)

type cOpsDevice struct{}

var terminalUpgrader = websocket.Upgrader{
	ReadBufferSize:  4096,
	WriteBufferSize: 4096,
	CheckOrigin: func(r *http.Request) bool {
		return true
	},
}

var desktopUpgrader = websocket.Upgrader{
	ReadBufferSize:  1024 * 1024,
	WriteBufferSize: 1024 * 1024,
	CheckOrigin: func(r *http.Request) bool {
		return true
	},
}

type wsFrame struct {
	messageType int
	payload     []byte
}

const (
	terminalWriteWait      = 10 * time.Second
	desktopWriteWait       = 10 * time.Second
	desktopTextQueueSize   = 32
	desktopVideoQueueSize  = 8
	weylusProxyPrefix      = "/admin/opsDevice/weylus"
	weylusLocalProxyPrefix = "/admin/opsDevice/weylusLocal"
	weylusLocalBase        = "http://127.0.0.1:1701"
)

var weylusLocalProxy = newWeylusProxy(weylusLocalProxyPrefix)

func newWeylusProxy(proxyPrefix string) *httputil.ReverseProxy {
	target, _ := url.Parse(weylusLocalBase)
	proxy := httputil.NewSingleHostReverseProxy(target)
	originalDirector := proxy.Director
	proxy.Director = func(req *http.Request) {
		originalDirector(req)
		req.URL.Scheme = target.Scheme
		req.URL.Host = target.Host
		req.Host = target.Host
		req.Header.Set("X-Forwarded-Host", req.Host)
	}
	proxy.ModifyResponse = func(res *http.Response) error {
		contentType := res.Header.Get("Content-Type")
		if !strings.Contains(contentType, "text/html") &&
			!strings.Contains(contentType, "javascript") &&
			!strings.HasSuffix(res.Request.URL.Path, "/lib.js") {
			return nil
		}
		body, err := io.ReadAll(res.Body)
		if err != nil {
			return err
		}
		_ = res.Body.Close()
		if strings.Contains(contentType, "text/html") {
			body = bytes.ReplaceAll(body, []byte(`href="style.css"`), []byte(fmt.Sprintf(`href="%s/style.css"`, proxyPrefix)))
			body = bytes.ReplaceAll(body, []byte(`src="lib.js"`), []byte(fmt.Sprintf(`src="%s/lib.js"`, proxyPrefix)))
			body = rewriteWeylusBranding(body)
		} else {
			body = bytes.ReplaceAll(body, []byte(`"/ws"`), []byte(fmt.Sprintf(`"%s/ws"`, proxyPrefix)))
		}
		res.Body = io.NopCloser(bytes.NewReader(body))
		res.ContentLength = int64(len(body))
		res.Header.Del("Content-Length")
		return nil
	}
	return proxy
}

func rewriteWeylusBranding(body []byte) []byte {
	replacements := [][2]string{
		{`<title>Weylus</title>`, `<title>AuroraOps 远程桌面</title>`},
		{`>Weylus<`, `>AuroraOps 远程桌面<`},
		{`Weylus`, `AuroraOps 远程桌面`},
	}
	for _, item := range replacements {
		body = bytes.ReplaceAll(body, []byte(item[0]), []byte(item[1]))
	}
	return body
}

func (c *cOpsDevice) List(ctx context.Context, req *opsdevice.ListReq) (res *opsdevice.ListRes, err error) {
	list, totalCount, err := service.SysOpsDevice().List(ctx, &req.OpsDeviceListInp)
	if err != nil {
		return
	}

	if list == nil {
		list = []*sysin.OpsDeviceListModel{}
	}

	res = new(opsdevice.ListRes)
	res.List = list
	res.PageRes.Pack(req, totalCount)
	return
}

func (c *cOpsDevice) Edit(ctx context.Context, req *opsdevice.EditReq) (res *opsdevice.EditRes, err error) {
	err = service.SysOpsDevice().Edit(ctx, &req.OpsDeviceEditInp)
	return
}

func (c *cOpsDevice) MaxSort(ctx context.Context, req *opsdevice.MaxSortReq) (res *opsdevice.MaxSortRes, err error) {
	data, err := service.SysOpsDevice().MaxSort(ctx, &req.OpsDeviceMaxSortInp)
	if err != nil {
		return
	}

	res = new(opsdevice.MaxSortRes)
	res.OpsDeviceMaxSortModel = data
	return
}

func (c *cOpsDevice) View(ctx context.Context, req *opsdevice.ViewReq) (res *opsdevice.ViewRes, err error) {
	data, err := service.SysOpsDevice().View(ctx, &req.OpsDeviceViewInp)
	if err != nil {
		return
	}

	res = new(opsdevice.ViewRes)
	res.OpsDeviceViewModel = data
	return
}

func (c *cOpsDevice) Delete(ctx context.Context, req *opsdevice.DeleteReq) (res *opsdevice.DeleteRes, err error) {
	err = service.SysOpsDevice().Delete(ctx, &req.OpsDeviceDeleteInp)
	return
}

func (c *cOpsDevice) Status(ctx context.Context, req *opsdevice.StatusReq) (res *opsdevice.StatusRes, err error) {
	err = service.SysOpsDevice().Status(ctx, &req.OpsDeviceStatusInp)
	return
}

func (c *cOpsDevice) Wake(ctx context.Context, req *opsdevice.WakeReq) (res *opsdevice.WakeRes, err error) {
	data, err := service.SysOpsDevice().Wake(ctx, &req.OpsDeviceWakeInp)
	if err != nil {
		return nil, err
	}
	res = new(opsdevice.WakeRes)
	res.OpsDeviceWakeModel = data
	return
}

func (c *cOpsDevice) Option(ctx context.Context, req *opsdevice.OptionReq) (res opsdevice.OptionRes, err error) {
	data, err := service.SysOpsDevice().Option(ctx)
	if err != nil {
		return nil, err
	}
	return opsdevice.OptionRes(data), nil
}

func (c *cOpsDevice) TerminalCreate(ctx context.Context, req *opsdevice.TerminalCreateReq) (res *opsdevice.TerminalCreateRes, err error) {
	data, err := service.SysOpsDevice().CreateTerminalSession(ctx, &req.OpsDeviceTerminalCreateInp)
	if err != nil {
		return
	}
	res = new(opsdevice.TerminalCreateRes)
	res.OpsDeviceTerminalCreateModel = data
	return
}

func (c *cOpsDevice) DesktopCreate(ctx context.Context, req *opsdevice.DesktopCreateReq) (res *opsdevice.DesktopCreateRes, err error) {
	data, err := service.SysOpsDevice().CreateDesktopSession(ctx, &req.OpsDeviceDesktopCreateInp)
	if err != nil {
		return
	}
	res = new(opsdevice.DesktopCreateRes)
	res.OpsDeviceDesktopCreateModel = data
	return
}

func (c *cOpsDevice) WeylusProxy(r *ghttp.Request) {
	deviceID := r.Get("deviceId").Uint64()
	if deviceID == 0 {
		r.Response.WriteStatusExit(http.StatusBadRequest, "deviceId is required")
		return
	}
	if tunnel := getWeylusTunnel(deviceID); tunnel != nil {
		proxyWeylusViaTunnel(r, tunnel)
		return
	}

	r.Response.WriteStatusExit(http.StatusBadGateway, "device weylus tunnel is not connected")
}

func (c *cOpsDevice) WeylusLocalProxy(r *ghttp.Request) {
	proxyRequest := r.Request.Clone(r.Context())
	proxyRequest.URL.Path = strings.TrimPrefix(proxyRequest.URL.Path, weylusLocalProxyPrefix)
	if proxyRequest.URL.Path == "" {
		proxyRequest.URL.Path = "/"
	}
	proxyRequest.URL.RawPath = ""
	proxyRequest.Host = "127.0.0.1:1701"
	proxyRequest.Header.Set("Host", "127.0.0.1:1701")
	weylusLocalProxy.ServeHTTP(r.Response.Writer, proxyRequest)
	r.Exit()
}

func IsWeylusPublicAsset(path string) bool {
	return path == weylusProxyPrefix+"/style.css" || path == weylusProxyPrefix+"/lib.js"
}

func (c *cOpsDevice) TerminalWS(r *ghttp.Request) {
	sessionID := r.Get("sessionId").String()
	if sessionID == "" {
		r.Response.WriteStatusExit(http.StatusBadRequest, "sessionId is required")
		return
	}

	ch, cancel, err := service.TCPServer().SubscribeTerminal(sessionID)
	if err != nil {
		r.Response.WriteStatusExit(http.StatusBadRequest, err.Error())
		return
	}
	defer cancel()

	conn, err := terminalUpgrader.Upgrade(r.Response.Writer, r.Request, nil)
	if err != nil {
		return
	}
	defer conn.Close()

	ctx, cancelCtx := context.WithCancel(r.Context())
	defer cancelCtx()
	writeDone := make(chan struct{})
	go func() {
		defer close(writeDone)
		for payload := range ch {
			_ = conn.SetWriteDeadline(time.Now().Add(terminalWriteWait))
			if writeErr := conn.WriteMessage(websocket.TextMessage, payload); writeErr != nil {
				return
			}
		}
	}()

	terminalOpened := false
	terminalClosedByUser := false
	openTerminal := func(cols, rows uint32, shell string) {
		if terminalOpened {
			if cols > 0 && rows > 0 {
				_ = service.TCPServer().SendTerminalResize(ctx, sessionID, cols, rows)
			}
			return
		}
		if cols == 0 {
			cols = 120
		}
		if rows == 0 {
			rows = 32
		}
		terminalOpened = true
		_ = service.TCPServer().SendTerminalOpen(ctx, sessionID, cols, rows, shell)
	}

readLoop:
	for {
		_, message, readErr := conn.ReadMessage()
		if readErr != nil {
			break
		}

		var payload struct {
			Type  string `json:"type"`
			Input string `json:"input"`
			Cols  uint32 `json:"cols"`
			Rows  uint32 `json:"rows"`
			Shell string `json:"shell"`
		}
		if err = json.Unmarshal(message, &payload); err != nil {
			continue
		}

		switch payload.Type {
		case "open":
			openTerminal(payload.Cols, payload.Rows, payload.Shell)
		case "input":
			if !terminalOpened {
				openTerminal(payload.Cols, payload.Rows, payload.Shell)
			}
			if payload.Input != "" {
				_ = service.TCPServer().SendTerminalInput(ctx, sessionID, payload.Input)
			}
		case "resize":
			openTerminal(payload.Cols, payload.Rows, payload.Shell)
		case "close":
			terminalClosedByUser = true
			break readLoop
		}
	}

	if terminalOpened {
		closeMessage := "terminal websocket disconnected"
		if terminalClosedByUser {
			closeMessage = "terminal closed by user"
		}
		_ = service.TCPServer().SendTerminalClose(ctx, sessionID, closeMessage)
	}
	cancelCtx()
	cancel()
	_ = conn.Close()
	<-writeDone
}

func (c *cOpsDevice) DesktopWS(r *ghttp.Request) {
	sessionID := r.Get("sessionId").String()
	if sessionID == "" {
		r.Response.WriteStatusExit(http.StatusBadRequest, "sessionId is required")
		return
	}

	ch, cancel, err := service.TCPServer().SubscribeDesktop(sessionID)
	if err != nil {
		r.Response.WriteStatusExit(http.StatusBadRequest, err.Error())
		return
	}
	defer cancel()

	conn, err := desktopUpgrader.Upgrade(r.Response.Writer, r.Request, nil)
	if err != nil {
		return
	}
	defer conn.Close()

	ctx, cancelCtx := context.WithCancel(r.Context())
	defer cancelCtx()
	textCh := make(chan wsFrame, desktopTextQueueSize)
	videoCh := make(chan wsFrame, desktopVideoQueueSize)
	writeDone := make(chan struct{})
	go func() {
		defer close(writeDone)
		writeFrame := func(frame wsFrame) bool {
			_ = conn.SetWriteDeadline(time.Now().Add(desktopWriteWait))
			if writeErr := conn.WriteMessage(frame.messageType, frame.payload); writeErr != nil {
				cancelCtx()
				_ = conn.Close()
				return false
			}
			return true
		}

		for {
			select {
			case frame, ok := <-textCh:
				if !ok {
					return
				}
				if !writeFrame(frame) {
					return
				}
				continue
			default:
			}

			select {
			case frame, ok := <-textCh:
				if !ok {
					return
				}
				if !writeFrame(frame) {
					return
				}
			case frame := <-videoCh:
				if !writeFrame(frame) {
					return
				}
			case <-ctx.Done():
				return
			}
		}
	}()

	enqueueReliable := func(frame wsFrame) bool {
		select {
		case textCh <- frame:
			return true
		case <-ctx.Done():
			return false
		case <-time.After(2 * time.Second):
			return false
		}
	}

	enqueueVideo := func(frame wsFrame) bool {
		select {
		case videoCh <- frame:
			return true
		default:
			select {
			case <-videoCh:
			default:
			}
			select {
			case videoCh <- frame:
				return true
			default:
				return false
			}
		}
	}

	go func() {
		defer cancelCtx()
		defer close(textCh)
		for {
			var payload []byte
			select {
			case <-ctx.Done():
				return
			case next, ok := <-ch:
				if !ok {
					return
				}
				payload = next
			}
			var frame struct {
				Type    string `json:"type"`
				Payload string `json:"payload"`
				Message string `json:"message"`
			}
			if unmarshalErr := json.Unmarshal(payload, &frame); unmarshalErr != nil {
				continue
			}
			switch frame.Type {
			case "text":
				if !enqueueReliable(wsFrame{messageType: websocket.TextMessage, payload: []byte(frame.Payload)}) {
					return
				}
			case "closed":
				if !enqueueReliable(wsFrame{messageType: websocket.TextMessage, payload: payload}) {
					return
				}
			case "binary":
				data, decodeErr := base64.StdEncoding.DecodeString(frame.Payload)
				if decodeErr != nil {
					continue
				}
				if !enqueueVideo(wsFrame{messageType: websocket.BinaryMessage, payload: data}) {
					return
				}
			}
		}
	}()

	_ = service.TCPServer().SendDesktopOpen(ctx, sessionID)

	for {
		select {
		case <-ctx.Done():
			_ = service.TCPServer().SendDesktopClose(ctx, sessionID, "desktop disconnected")
			<-writeDone
			return
		default:
		}
		messageType, message, readErr := conn.ReadMessage()
		if readErr != nil {
			break
		}
		switch messageType {
		case websocket.TextMessage:
			_ = service.TCPServer().SendDesktopText(ctx, sessionID, string(message))
		case websocket.BinaryMessage:
			_ = service.TCPServer().SendDesktopBinary(ctx, sessionID, message)
		case websocket.CloseMessage:
			_ = service.TCPServer().SendDesktopClose(ctx, sessionID, "desktop closed by user")
			return
		}
	}

	_ = service.TCPServer().SendDesktopClose(ctx, sessionID, "desktop disconnected")
	cancelCtx()
	<-writeDone
}
