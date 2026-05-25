package tcpserver

import (
	"context"
	"encoding/json"
	"testing"
	"time"

	"auroraops/internal/library/network/tcp"
)

func TestDesktopHubBroadcastAndClose(t *testing.T) {
	hub := newDesktopHub()
	client, cancel := hub.subscribe("session-1")
	defer cancel()

	hub.broadcast("session-1", []byte(`{"type":"text","payload":"ok"}`))

	select {
	case got := <-client.send:
		if string(got) != `{"type":"text","payload":"ok"}` {
			t.Fatalf("unexpected payload: %s", got)
		}
	case <-time.After(time.Second):
		t.Fatal("timed out waiting for broadcast")
	}

	hub.close("session-1")
	select {
	case _, ok := <-client.send:
		if ok {
			t.Fatal("expected desktop client channel to close")
		}
	case <-time.After(time.Second):
		t.Fatal("timed out waiting for channel close")
	}
}

func TestDesktopManagerLifecycle(t *testing.T) {
	manager := newDesktopManager()
	session := manager.create(1001, 2002)
	if session.ID == "" {
		t.Fatal("expected session id")
	}
	if session.DeviceID != 1001 || session.CreatedBy != 2002 {
		t.Fatalf("unexpected session metadata: %#v", session)
	}

	got, ok := manager.get(session.ID)
	if !ok || got.ID != session.ID {
		t.Fatal("expected session to be readable")
	}

	manager.touch(session.ID)
	got, ok = manager.get(session.ID)
	if !ok {
		t.Fatal("expected touched session")
	}
	if !got.ExpiresAt.After(time.Now()) {
		t.Fatalf("expected future expiry, got %s", got.ExpiresAt)
	}

	manager.delete(session.ID)
	if _, ok = manager.get(session.ID); ok {
		t.Fatal("expected deleted session to be absent")
	}
}

func TestDesktopOutputHandlersBroadcastFrames(t *testing.T) {
	server := newTCPServer()
	session := server.desktops.create(1001, 2002)
	ch, cancel, err := server.SubscribeDesktop(session.ID)
	if err != nil {
		t.Fatalf("subscribe failed: %v", err)
	}
	defer cancel()

	server.onDeviceDesktopTextOutput(context.Background(), &tcp.DeviceDesktopTextOutputReq{
		SessionId: session.ID,
		Payload:   `{"kind":"Config"}`,
	})

	var frame struct {
		Type      string `json:"type"`
		SessionID string `json:"sessionId"`
		Payload   string `json:"payload"`
	}
	select {
	case payload := <-ch:
		if err = json.Unmarshal(payload, &frame); err != nil {
			t.Fatalf("unmarshal frame: %v", err)
		}
	case <-time.After(time.Second):
		t.Fatal("timed out waiting for text frame")
	}
	if frame.Type != "text" || frame.SessionID != session.ID || frame.Payload != `{"kind":"Config"}` {
		t.Fatalf("unexpected text frame: %#v", frame)
	}

	server.onDeviceDesktopBinaryOutput(context.Background(), &tcp.DeviceDesktopBinaryOutputReq{
		SessionId: session.ID,
		Payload:   "AQID",
	})
	select {
	case payload := <-ch:
		if err = json.Unmarshal(payload, &frame); err != nil {
			t.Fatalf("unmarshal binary frame: %v", err)
		}
	case <-time.After(time.Second):
		t.Fatal("timed out waiting for binary frame")
	}
	if frame.Type != "binary" || frame.Payload != "AQID" {
		t.Fatalf("unexpected binary frame: %#v", frame)
	}

	server.onDeviceDesktopClosed(context.Background(), &tcp.DeviceDesktopClosedReq{
		SessionId: session.ID,
		Message:   "done",
	})
	select {
	case payload, ok := <-ch:
		if !ok {
			t.Fatal("expected closed frame before channel close")
		}
		if err = json.Unmarshal(payload, &frame); err != nil {
			t.Fatalf("unmarshal closed frame: %v", err)
		}
	case <-time.After(time.Second):
		t.Fatal("timed out waiting for closed frame")
	}
	if frame.Type != "closed" {
		t.Fatalf("unexpected closed frame: %#v", frame)
	}
}
