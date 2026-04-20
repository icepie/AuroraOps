package main

import (
	"encoding/json"
	"errors"
	"fmt"
	"log"
	"os"
	"os/exec"
	"runtime"
	"strings"
	"sync"

	"github.com/creack/pty"
)

type DeviceTerminalOpenRequest struct {
	SessionID string `json:"sessionId"`
	Cols      uint32 `json:"cols"`
	Rows      uint32 `json:"rows"`
	Shell     string `json:"shell,omitempty"`
}

type DeviceTerminalInputRequest struct {
	SessionID string `json:"sessionId"`
	Input     string `json:"input"`
}

type DeviceTerminalResizeRequest struct {
	SessionID string `json:"sessionId"`
	Cols      uint32 `json:"cols"`
	Rows      uint32 `json:"rows"`
}

type DeviceTerminalCloseRequest struct {
	SessionID string `json:"sessionId"`
	Message   string `json:"message,omitempty"`
}

type DeviceTerminalOutputRequest struct {
	SessionID string `json:"sessionId"`
	Output    string `json:"output"`
}

type DeviceTerminalClosedRequest struct {
	SessionID string `json:"sessionId"`
	Message   string `json:"message,omitempty"`
}

type TerminalManager struct {
	mu       sync.RWMutex
	sessions map[string]*TerminalSession
	tcp      *AgentTCPClient
}

type TerminalSession struct {
	id   string
	tty  *os.File
	cmd  *exec.Cmd
	done chan struct{}
}

func NewTerminalManager(tcpClient *AgentTCPClient) *TerminalManager {
	return &TerminalManager{
		sessions: make(map[string]*TerminalSession),
		tcp:      tcpClient,
	}
}

func (m *TerminalManager) HandleEnvelope(envelope TCPEnvelope) {
	switch envelope.Router {
	case "DeviceTerminalOpenReq":
		var req DeviceTerminalOpenRequest
		if err := json.Unmarshal(envelope.Data, &req); err != nil {
			log.Printf("terminal open decode failed: %v\n", err)
			return
		}
		if err := m.Open(req); err != nil {
			_ = m.tcp.Send("DeviceTerminalClosedReq", DeviceTerminalClosedRequest{
				SessionID: req.SessionID,
				Message:   err.Error(),
			})
		}
	case "DeviceTerminalInputReq":
		var req DeviceTerminalInputRequest
		if err := json.Unmarshal(envelope.Data, &req); err != nil {
			return
		}
		if err := m.Input(req); err != nil {
			_ = m.tcp.Send("DeviceTerminalClosedReq", DeviceTerminalClosedRequest{
				SessionID: req.SessionID,
				Message:   err.Error(),
			})
		}
	case "DeviceTerminalResizeReq":
		var req DeviceTerminalResizeRequest
		if err := json.Unmarshal(envelope.Data, &req); err != nil {
			return
		}
		if err := m.Resize(req); err != nil {
			log.Printf("terminal resize failed: %v\n", err)
		}
	case "DeviceTerminalCloseReq":
		var req DeviceTerminalCloseRequest
		if err := json.Unmarshal(envelope.Data, &req); err != nil {
			return
		}
		m.Close(req.SessionID, firstNonEmpty(req.Message, "terminal closed by server"))
	}
}

func (m *TerminalManager) Open(req DeviceTerminalOpenRequest) error {
	if strings.TrimSpace(req.SessionID) == "" {
		return errors.New("terminal sessionId is required")
	}

	m.mu.Lock()
	if _, exists := m.sessions[req.SessionID]; exists {
		m.mu.Unlock()
		return errors.New("terminal session already exists")
	}

	cmd, err := buildTerminalCommand(req.Shell)
	if err != nil {
		m.mu.Unlock()
		return err
	}

	if runtime.GOOS != "windows" {
		cmd.Env = append(os.Environ(), "TERM=xterm")
	}

	tty, err := pty.Start(cmd)
	if err != nil {
		m.mu.Unlock()
		return fmt.Errorf("start pty: %w", err)
	}

	session := &TerminalSession{
		id:   req.SessionID,
		tty:  tty,
		cmd:  cmd,
		done: make(chan struct{}),
	}
	m.sessions[req.SessionID] = session
	m.mu.Unlock()

	if req.Cols > 0 && req.Rows > 0 {
		_ = pty.Setsize(tty, &pty.Winsize{
			Cols: uint16(req.Cols),
			Rows: uint16(req.Rows),
		})
	}

	go m.streamOutput(session)
	return nil
}

func (m *TerminalManager) Input(req DeviceTerminalInputRequest) error {
	session, err := m.getSession(req.SessionID)
	if err != nil {
		return err
	}
	_, err = session.tty.WriteString(req.Input)
	return err
}

func (m *TerminalManager) Resize(req DeviceTerminalResizeRequest) error {
	session, err := m.getSession(req.SessionID)
	if err != nil {
		return err
	}
	if req.Cols == 0 || req.Rows == 0 {
		return nil
	}
	return pty.Setsize(session.tty, &pty.Winsize{
		Cols: uint16(req.Cols),
		Rows: uint16(req.Rows),
	})
}

func (m *TerminalManager) Close(sessionID, message string) {
	session, ok := m.deleteSession(sessionID)
	if !ok {
		return
	}
	close(session.done)
	_ = session.tty.Close()
	if session.cmd != nil && session.cmd.Process != nil {
		_ = session.cmd.Process.Kill()
		_, _ = session.cmd.Process.Wait()
	}
	if message != "" {
		_ = m.tcp.Send("DeviceTerminalClosedReq", DeviceTerminalClosedRequest{
			SessionID: sessionID,
			Message:   message,
		})
	}
}

func (m *TerminalManager) CloseAll() {
	m.mu.Lock()
	ids := make([]string, 0, len(m.sessions))
	for id := range m.sessions {
		ids = append(ids, id)
	}
	m.mu.Unlock()

	for _, id := range ids {
		m.Close(id, "agent connection closed")
	}
}

func (m *TerminalManager) streamOutput(session *TerminalSession) {
	buf := make([]byte, 4096)
	for {
		n, err := session.tty.Read(buf)
		if n > 0 {
			if sendErr := m.tcp.Send("DeviceTerminalOutputReq", DeviceTerminalOutputRequest{
				SessionID: session.id,
				Output:    string(buf[:n]),
			}); sendErr != nil {
				log.Printf("terminal output send failed: %v\n", sendErr)
				break
			}
		}
		if err != nil {
			break
		}
		select {
		case <-session.done:
			return
		default:
		}
	}

	m.Close(session.id, "terminal exited")
}

func (m *TerminalManager) getSession(sessionID string) (*TerminalSession, error) {
	m.mu.RLock()
	defer m.mu.RUnlock()
	session, ok := m.sessions[sessionID]
	if !ok {
		return nil, errors.New("terminal session not found")
	}
	return session, nil
}

func (m *TerminalManager) deleteSession(sessionID string) (*TerminalSession, bool) {
	m.mu.Lock()
	defer m.mu.Unlock()
	session, ok := m.sessions[sessionID]
	if ok {
		delete(m.sessions, sessionID)
	}
	return session, ok
}

func buildTerminalCommand(shell string) (*exec.Cmd, error) {
	candidates := make([]string, 0, 5)
	if strings.TrimSpace(shell) != "" {
		candidates = append(candidates, strings.TrimSpace(shell))
	}
	if runtime.GOOS == "windows" {
		candidates = append(candidates, "pwsh.exe", "powershell.exe", "cmd.exe")
	} else {
		candidates = append(candidates, "bash", "zsh", "sh")
	}

	for _, candidate := range candidates {
		path, err := exec.LookPath(candidate)
		if err == nil && path != "" {
			return exec.Command(path), nil // #nosec G204
		}
	}
	return nil, errors.New("no terminal shell available")
}
