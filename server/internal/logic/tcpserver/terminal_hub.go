package tcpserver

import (
	"sync"
)

type terminalClient struct {
	send chan []byte
	once sync.Once
}

type terminalHub struct {
	mu      sync.RWMutex
	clients map[string]map[*terminalClient]struct{}
}

func newTerminalHub() *terminalHub {
	return &terminalHub{
		clients: make(map[string]map[*terminalClient]struct{}),
	}
}

func (h *terminalHub) add(sessionID string, client *terminalClient) {
	h.mu.Lock()
	if _, ok := h.clients[sessionID]; !ok {
		h.clients[sessionID] = make(map[*terminalClient]struct{})
	}
	h.clients[sessionID][client] = struct{}{}
	h.mu.Unlock()
}

func (h *terminalHub) subscribe(sessionID string) (*terminalClient, func()) {
	client := &terminalClient{
		send: make(chan []byte, 256),
	}
	h.add(sessionID, client)
	cancel := func() {
		h.remove(sessionID, client)
	}
	return client, cancel
}

func (h *terminalHub) remove(sessionID string, client *terminalClient) {
	h.mu.Lock()
	if clients, ok := h.clients[sessionID]; ok {
		delete(clients, client)
		if len(clients) == 0 {
			delete(h.clients, sessionID)
		}
	}
	h.mu.Unlock()
	client.once.Do(func() {
		close(client.send)
	})
}

func (h *terminalHub) broadcast(sessionID string, payload []byte) {
	h.mu.RLock()
	clients := h.clients[sessionID]
	h.mu.RUnlock()
	for client := range clients {
		select {
		case client.send <- payload:
		default:
		}
	}
}

func (h *terminalHub) close(sessionID string) {
	h.mu.Lock()
	clients := h.clients[sessionID]
	delete(h.clients, sessionID)
	h.mu.Unlock()
	for client := range clients {
		client.once.Do(func() {
			close(client.send)
		})
	}
}
