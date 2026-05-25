package tcpserver

import (
	"context"
	"encoding/base64"
	"encoding/json"
	"errors"
	"sync"
	"time"

	"github.com/gogf/gf/v2/frame/g"
	"github.com/gogf/gf/v2/util/guid"

	"auroraops/internal/library/network/tcp"
)

const desktopSessionTTL = 15 * time.Minute

type desktopSession struct {
	ID        string
	DeviceID  uint64
	CreatedBy int64
	CreatedAt time.Time
	ExpiresAt time.Time
}

type desktopManager struct {
	mu       sync.RWMutex
	sessions map[string]*desktopSession
}

func newDesktopManager() *desktopManager {
	return &desktopManager{
		sessions: make(map[string]*desktopSession),
	}
}

func (m *desktopManager) create(deviceID uint64, userID int64) *desktopSession {
	session := &desktopSession{
		ID:        guid.S(),
		DeviceID:  deviceID,
		CreatedBy: userID,
		CreatedAt: time.Now(),
		ExpiresAt: time.Now().Add(desktopSessionTTL),
	}
	m.mu.Lock()
	m.sessions[session.ID] = session
	m.mu.Unlock()
	return session
}

func (m *desktopManager) get(sessionID string) (*desktopSession, bool) {
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

func (m *desktopManager) delete(sessionID string) {
	m.mu.Lock()
	delete(m.sessions, sessionID)
	m.mu.Unlock()
}

func (m *desktopManager) touch(sessionID string) {
	m.mu.Lock()
	if session, ok := m.sessions[sessionID]; ok {
		session.ExpiresAt = time.Now().Add(desktopSessionTTL)
	}
	m.mu.Unlock()
}

func (m *desktopManager) cleanupLoop(ctx context.Context) {
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

func (s *sTCPServer) CreateDesktopSession(ctx context.Context, deviceId uint64, userId int64) (sessionId string, err error) {
	if deviceId == 0 {
		return "", errors.New("设备ID不能为空")
	}
	if userId == 0 {
		return "", errors.New("用户未登录")
	}
	if len(s.getDeviceClients(deviceId)) == 0 {
		return "", errors.New("设备离线，无法发起远程桌面")
	}
	session := s.desktops.create(deviceId, userId)
	return session.ID, nil
}

func (s *sTCPServer) SendDesktopOpen(ctx context.Context, sessionId string) error {
	return s.forwardDesktopMessage(ctx, sessionId, &tcp.DeviceDesktopOpenReq{
		SessionId: sessionId,
	})
}

func (s *sTCPServer) SendDesktopText(ctx context.Context, sessionId, payload string) error {
	return s.forwardDesktopMessage(ctx, sessionId, &tcp.DeviceDesktopTextReq{
		SessionId: sessionId,
		Payload:   payload,
	})
}

func (s *sTCPServer) SendDesktopBinary(ctx context.Context, sessionId string, payload []byte) error {
	return s.forwardDesktopMessage(ctx, sessionId, &tcp.DeviceDesktopBinaryReq{
		SessionId: sessionId,
		Payload:   base64.StdEncoding.EncodeToString(payload),
	})
}

func (s *sTCPServer) SendDesktopClose(ctx context.Context, sessionId, message string) error {
	return s.forwardDesktopMessage(ctx, sessionId, &tcp.DeviceDesktopCloseReq{
		SessionId: sessionId,
		Message:   message,
	})
}

func (s *sTCPServer) SubscribeDesktop(sessionId string) (ch <-chan []byte, cancel func(), err error) {
	if _, ok := s.desktops.get(sessionId); !ok {
		return nil, nil, errors.New("远程桌面会话不存在或已过期")
	}
	client, release := s.desktopHub.subscribe(sessionId)
	return client.send, release, nil
}

func (s *sTCPServer) forwardDesktopMessage(ctx context.Context, sessionId string, payload interface{}) error {
	session, ok := s.desktops.get(sessionId)
	if !ok {
		return errors.New("远程桌面会话不存在或已过期")
	}
	clients := s.getDeviceClients(session.DeviceID)
	if len(clients) == 0 {
		return errors.New("设备离线，无法发送远程桌面指令")
	}
	s.desktops.touch(sessionId)
	for _, client := range clients {
		if client == nil {
			continue
		}
		if err := client.Send(ctx, payload); err == nil {
			return nil
		}
	}
	return errors.New("远程桌面指令下发失败")
}

func (s *sTCPServer) onDeviceDesktopTextOutput(ctx context.Context, req *tcp.DeviceDesktopTextOutputReq) {
	if req == nil || req.SessionId == "" {
		return
	}
	if _, ok := s.desktops.get(req.SessionId); !ok {
		return
	}
	s.desktops.touch(req.SessionId)
	payload, err := json.Marshal(g.Map{
		"type":      "text",
		"sessionId": req.SessionId,
		"payload":   req.Payload,
	})
	if err == nil {
		s.desktopHub.broadcast(req.SessionId, payload)
	}
}

func (s *sTCPServer) onDeviceDesktopBinaryOutput(ctx context.Context, req *tcp.DeviceDesktopBinaryOutputReq) {
	if req == nil || req.SessionId == "" {
		return
	}
	if _, ok := s.desktops.get(req.SessionId); !ok {
		return
	}
	s.desktops.touch(req.SessionId)
	payload, err := json.Marshal(g.Map{
		"type":      "binary",
		"sessionId": req.SessionId,
		"payload":   req.Payload,
	})
	if err == nil {
		s.desktopHub.broadcast(req.SessionId, payload)
	}
}

func (s *sTCPServer) onDeviceDesktopClosed(ctx context.Context, req *tcp.DeviceDesktopClosedReq) {
	if req == nil || req.SessionId == "" {
		return
	}
	payload, err := json.Marshal(g.Map{
		"type":      "closed",
		"sessionId": req.SessionId,
		"message":   req.Message,
	})
	if err == nil {
		s.desktopHub.broadcast(req.SessionId, payload)
	}
	s.desktops.delete(req.SessionId)
	s.desktopHub.close(req.SessionId)
}
