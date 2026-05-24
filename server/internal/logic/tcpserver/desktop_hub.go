package tcpserver

import "sync"

type desktopClient struct {
	send chan []byte
	once sync.Once
}

type desktopHub struct {
	mu      sync.RWMutex
	clients map[string]map[*desktopClient]struct{}
}

func newDesktopHub() *desktopHub {
	return &desktopHub{
		clients: make(map[string]map[*desktopClient]struct{}),
	}
}

func (h *desktopHub) add(sessionID string, client *desktopClient) {
	h.mu.Lock()
	if _, ok := h.clients[sessionID]; !ok {
		h.clients[sessionID] = make(map[*desktopClient]struct{})
	}
	h.clients[sessionID][client] = struct{}{}
	h.mu.Unlock()
}

func (h *desktopHub) subscribe(sessionID string) (*desktopClient, func()) {
	client := &desktopClient{
		send: make(chan []byte, 256),
	}
	h.add(sessionID, client)
	cancel := func() {
		h.remove(sessionID, client)
	}
	return client, cancel
}

func (h *desktopHub) remove(sessionID string, client *desktopClient) {
	removed := false
	h.mu.Lock()
	if clients, ok := h.clients[sessionID]; ok {
		if _, ok = clients[client]; ok {
			delete(clients, client)
			removed = true
		}
		if len(clients) == 0 {
			delete(h.clients, sessionID)
		}
	}
	h.mu.Unlock()
	if removed {
		client.close()
	}
}

func (h *desktopHub) broadcast(sessionID string, payload []byte) {
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

func (h *desktopHub) close(sessionID string) {
	h.mu.Lock()
	clients := h.clients[sessionID]
	delete(h.clients, sessionID)
	h.mu.Unlock()
	for client := range clients {
		client.close()
	}
}

func (c *desktopClient) close() {
	c.once.Do(func() {
		close(c.send)
	})
}
