package main

import (
	"encoding/binary"
	"encoding/json"
	"errors"
	"io"
	"net"
	"sync"
	"time"
)

type AgentTCPClient struct {
	conn    net.Conn
	writeMu sync.Mutex

	pendingMu sync.Mutex
	pending   map[string]chan json.RawMessage

	handlerMu sync.RWMutex
	handler   func(TCPEnvelope)

	closeOnce sync.Once
	closed    chan struct{}
}

func NewAgentTCPClient(conn net.Conn) *AgentTCPClient {
	return &AgentTCPClient{
		conn:    conn,
		pending: make(map[string]chan json.RawMessage),
		closed:  make(chan struct{}),
	}
}

func (c *AgentTCPClient) SetHandler(handler func(TCPEnvelope)) {
	c.handlerMu.Lock()
	defer c.handlerMu.Unlock()
	c.handler = handler
}

func (c *AgentTCPClient) Start() {
	go c.readLoop()
}

func (c *AgentTCPClient) Close() error {
	var err error
	c.closeOnce.Do(func() {
		close(c.closed)
		err = c.conn.Close()

		c.pendingMu.Lock()
		defer c.pendingMu.Unlock()
		for key, ch := range c.pending {
			close(ch)
			delete(c.pending, key)
		}
	})
	return err
}

func (c *AgentTCPClient) Send(router string, data any) error {
	msg := TCPMessage{Router: router, Data: data}
	b, err := json.Marshal(msg)
	if err != nil {
		return err
	}
	if len(b) > 0xFFFF {
		return errors.New("tcp payload too large")
	}

	packet := make([]byte, 2+len(b))
	binary.BigEndian.PutUint16(packet[:2], uint16(len(b)))
	copy(packet[2:], b)

	c.writeMu.Lock()
	defer c.writeMu.Unlock()
	_, err = c.conn.Write(packet)
	return err
}

func (c *AgentTCPClient) Request(expected string, timeout time.Duration, router string, data any) (json.RawMessage, error) {
	ch := make(chan json.RawMessage, 1)

	c.pendingMu.Lock()
	if _, exists := c.pending[expected]; exists {
		c.pendingMu.Unlock()
		return nil, errors.New("duplicate pending tcp response: " + expected)
	}
	c.pending[expected] = ch
	c.pendingMu.Unlock()

	if err := c.Send(router, data); err != nil {
		c.pendingMu.Lock()
		delete(c.pending, expected)
		c.pendingMu.Unlock()
		return nil, err
	}

	timer := time.NewTimer(timeout)
	defer timer.Stop()

	select {
	case <-c.closed:
		return nil, errors.New("tcp connection closed")
	case payload, ok := <-ch:
		if !ok {
			return nil, errors.New("tcp connection closed")
		}
		return payload, nil
	case <-timer.C:
		c.pendingMu.Lock()
		delete(c.pending, expected)
		c.pendingMu.Unlock()
		return nil, errors.New("tcp request timeout: " + expected)
	}
}

func (c *AgentTCPClient) readLoop() {
	defer c.Close()

	for {
		packet, err := recvTCPPacket(c.conn)
		if err != nil {
			if !errors.Is(err, io.EOF) {
				// fall through to close the connection and pending waiters
			}
			return
		}
		if len(packet) == 0 {
			continue
		}

		var envelope TCPEnvelope
		if err = json.Unmarshal(packet, &envelope); err != nil {
			continue
		}

		if c.dispatchPending(envelope) {
			continue
		}

		c.handlerMu.RLock()
		handler := c.handler
		c.handlerMu.RUnlock()
		if handler != nil {
			handler(envelope)
		}
	}
}

func (c *AgentTCPClient) dispatchPending(envelope TCPEnvelope) bool {
	c.pendingMu.Lock()
	ch, ok := c.pending[envelope.Router]
	if ok {
		delete(c.pending, envelope.Router)
	}
	c.pendingMu.Unlock()

	if !ok {
		return false
	}

	select {
	case ch <- envelope.Data:
	default:
	}
	close(ch)
	return true
}
