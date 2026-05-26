package tcpserver

import (
	"fmt"

	"github.com/gogf/gf/v2/os/gtime"
	"github.com/gogf/gf/v2/util/gconv"

	"auroraops/internal/library/network/tcp"
)

func (s *sTCPServer) getDeviceClients(deviceID uint64) []*tcp.Conn {
	return s.getDeviceClientsWithFreshness(deviceID, true)
}

func (s *sTCPServer) getDeviceClientsWithFreshness(deviceID uint64, requireFreshHeartbeat bool) []*tcp.Conn {
	if deviceID == 0 || s.serv == nil {
		return nil
	}
	clients := s.serv.GetAppIdClients(fmt.Sprintf("device:%d", deviceID))
	if len(clients) > 0 {
		online := make([]*tcp.Conn, 0, len(clients))
		for _, client := range clients {
			if s.isFreshDeviceClient(client, requireFreshHeartbeat) {
				online = append(online, client)
			}
		}
		return online
	}

	groupClients := s.serv.GetGroupClients("device")
	matched := make([]*tcp.Conn, 0, len(groupClients))
	for _, client := range groupClients {
		if !s.isFreshDeviceClient(client, requireFreshHeartbeat) || client.Auth.Extra == nil {
			continue
		}
		if gconv.Uint64(client.Auth.Extra["deviceId"]) == deviceID {
			matched = append(matched, client)
		}
	}
	return matched
}

func (s *sTCPServer) isFreshDeviceClient(client *tcp.Conn, requireFreshHeartbeat bool) bool {
	if client == nil || client.IsClosed() || client.Auth == nil || client.Auth.Group != "device" {
		return false
	}
	if !requireFreshHeartbeat {
		return true
	}
	return client.Heartbeat >= gtime.Timestamp()-tcp.HeartbeatOnlineWindow
}

func (s *sTCPServer) OnlineDeviceIDs() map[uint64]struct{} {
	onlineSet := make(map[uint64]struct{})
	if s.serv == nil {
		return onlineSet
	}
	for _, client := range s.serv.GetGroupClients("device") {
		if !s.isFreshDeviceClient(client, true) {
			continue
		}
		if deviceID := parseDeviceIDFromClient(client); deviceID > 0 {
			onlineSet[deviceID] = struct{}{}
		}
	}
	return onlineSet
}

func (s *sTCPServer) IsDeviceOnline(deviceID uint64) bool {
	return len(s.getDeviceClients(deviceID)) > 0
}

func parseDeviceIDFromClient(client *tcp.Conn) uint64 {
	if client == nil || client.Auth == nil {
		return 0
	}
	if deviceID := parseDeviceIDFromAppID(client.Auth.AppId); deviceID > 0 {
		return deviceID
	}
	if client.Auth.Extra == nil {
		return 0
	}
	return gconv.Uint64(client.Auth.Extra["deviceId"])
}

func parseDeviceIDFromAppID(appID string) uint64 {
	var deviceID uint64
	if _, err := fmt.Sscanf(appID, "device:%d", &deviceID); err != nil {
		return 0
	}
	return deviceID
}
