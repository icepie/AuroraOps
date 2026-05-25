package tcpserver

import (
	"context"
	"encoding/json"
	"errors"
	"sync"
	"time"

	"github.com/gogf/gf/v2/frame/g"
	"github.com/gogf/gf/v2/util/guid"

	"auroraops/internal/library/network/tcp"
)

const terminalSessionTTL = 15 * time.Minute
const terminalSessionReuseWindow = 30 * time.Second

type terminalSession struct {
	ID        string
	DeviceID  uint64
	CreatedBy int64
	CreatedAt time.Time
	ExpiresAt time.Time
}

type terminalManager struct {
	mu       sync.RWMutex
	sessions map[string]*terminalSession
}

func newTerminalManager() *terminalManager {
	return &terminalManager{
		sessions: make(map[string]*terminalSession),
	}
}

func (m *terminalManager) create(deviceID uint64, userID int64) *terminalSession {
	now := time.Now()
	m.mu.Lock()
	for _, session := range m.sessions {
		if session.DeviceID == deviceID &&
			session.CreatedBy == userID &&
			now.Before(session.ExpiresAt) &&
			now.Sub(session.CreatedAt) <= terminalSessionReuseWindow {
			session.ExpiresAt = now.Add(terminalSessionTTL)
			m.mu.Unlock()
			return session
		}
	}
	session := &terminalSession{
		ID:        guid.S(),
		DeviceID:  deviceID,
		CreatedBy: userID,
		CreatedAt: now,
		ExpiresAt: now.Add(terminalSessionTTL),
	}
	m.sessions[session.ID] = session
	m.mu.Unlock()
	return session
}

func (m *terminalManager) get(sessionID string) (*terminalSession, bool) {
	m.mu.RLock()
	session, ok := m.sessions[sessionID]
	m.mu.RUnlock()
	if !ok {
		return nil, false
	}
	if time.Now().After(session.ExpiresAt) {
		m.delete(sessionID)
		return nil, false
	}
	return session, true
}

func (m *terminalManager) delete(sessionID string) {
	m.mu.Lock()
	delete(m.sessions, sessionID)
	m.mu.Unlock()
}

func (m *terminalManager) touch(sessionID string) {
	m.mu.Lock()
	if session, ok := m.sessions[sessionID]; ok {
		session.ExpiresAt = time.Now().Add(terminalSessionTTL)
	}
	m.mu.Unlock()
}

func (m *terminalManager) cleanupLoop(ctx context.Context) {
	ticker := time.NewTicker(time.Minute)
	defer ticker.Stop()
	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			now := time.Now()
			m.mu.Lock()
			for id, session := range m.sessions {
				if now.After(session.ExpiresAt) {
					delete(m.sessions, id)
				}
			}
			m.mu.Unlock()
		}
	}
}

func (s *sTCPServer) CreateTerminalSession(ctx context.Context, deviceId uint64, userId int64) (sessionId string, err error) {
	if deviceId == 0 {
		return "", errors.New("设备ID不能为空")
	}
	if userId == 0 {
		return "", errors.New("用户未登录")
	}
	if len(s.getDeviceClients(deviceId)) == 0 {
		return "", errors.New("设备离线，无法发起远程登录")
	}
	session := s.terminals.create(deviceId, userId)
	return session.ID, nil
}

func (s *sTCPServer) SendTerminalOpen(ctx context.Context, sessionId string, cols, rows uint32, shell string) error {
	return s.forwardTerminalMessage(ctx, sessionId, &tcp.DeviceTerminalOpenReq{
		SessionId: sessionId,
		Cols:      cols,
		Rows:      rows,
		Shell:     shell,
	})
}

func (s *sTCPServer) SendTerminalInput(ctx context.Context, sessionId, input string) error {
	return s.forwardTerminalMessage(ctx, sessionId, &tcp.DeviceTerminalInputReq{
		SessionId: sessionId,
		Input:     input,
	})
}

func (s *sTCPServer) SendTerminalResize(ctx context.Context, sessionId string, cols, rows uint32) error {
	return s.forwardTerminalMessage(ctx, sessionId, &tcp.DeviceTerminalResizeReq{
		SessionId: sessionId,
		Cols:      cols,
		Rows:      rows,
	})
}

func (s *sTCPServer) SendTerminalClose(ctx context.Context, sessionId, message string) error {
	return s.forwardTerminalMessage(ctx, sessionId, &tcp.DeviceTerminalCloseReq{
		SessionId: sessionId,
		Message:   message,
	})
}

func (s *sTCPServer) SubscribeTerminal(sessionId string) (ch <-chan []byte, cancel func(), err error) {
	if _, ok := s.terminals.get(sessionId); !ok {
		return nil, nil, errors.New("终端会话不存在或已过期")
	}
	client, release := s.terminalHub.subscribe(sessionId)
	return client.send, release, nil
}

func (s *sTCPServer) forwardTerminalMessage(ctx context.Context, sessionId string, payload interface{}) error {
	session, ok := s.terminals.get(sessionId)
	if !ok {
		return errors.New("终端会话不存在或已过期")
	}
	clients := s.getDeviceClients(session.DeviceID)
	if len(clients) == 0 {
		return errors.New("设备离线，无法发送终端指令")
	}
	s.terminals.touch(sessionId)
	for _, client := range clients {
		if client == nil {
			continue
		}
		if err := client.Send(ctx, payload); err == nil {
			return nil
		}
	}
	return errors.New("终端指令下发失败")
}

func (s *sTCPServer) onDeviceTerminalOutput(ctx context.Context, req *tcp.DeviceTerminalOutputReq) {
	if req == nil || req.SessionId == "" {
		return
	}
	session, ok := s.terminals.get(req.SessionId)
	if !ok {
		return
	}
	s.terminals.touch(req.SessionId)
	payload, err := json.Marshal(g.Map{
		"type":      "output",
		"sessionId": req.SessionId,
		"output":    req.Output,
	})
	if err != nil {
		return
	}
	s.terminalHub.broadcast(session.ID, payload)
}

func (s *sTCPServer) onDeviceTerminalClosed(ctx context.Context, req *tcp.DeviceTerminalClosedReq) {
	if req == nil || req.SessionId == "" {
		return
	}
	if _, ok := s.terminals.get(req.SessionId); !ok {
		return
	}
	payload, err := json.Marshal(g.Map{
		"type":      "closed",
		"sessionId": req.SessionId,
		"message":   req.Message,
	})
	if err == nil {
		s.terminalHub.broadcast(req.SessionId, payload)
	}
	s.terminals.delete(req.SessionId)
	s.terminalHub.close(req.SessionId)
}
