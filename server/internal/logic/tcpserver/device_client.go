package tcpserver

import (
	"fmt"

	"github.com/gogf/gf/v2/util/gconv"

	"auroraops/internal/library/network/tcp"
)

func (s *sTCPServer) getDeviceClients(deviceID uint64) []*tcp.Conn {
	if deviceID == 0 || s.serv == nil {
		return nil
	}
	clients := s.serv.GetAppIdClients(fmt.Sprintf("device:%d", deviceID))
	if len(clients) > 0 {
		online := make([]*tcp.Conn, 0, len(clients))
		for _, client := range clients {
			if client != nil && !client.IsClosed() {
				online = append(online, client)
			}
		}
		return online
	}

	groupClients := s.serv.GetGroupClients("device")
	matched := make([]*tcp.Conn, 0, len(groupClients))
	for _, client := range groupClients {
		if client == nil || client.IsClosed() || client.Auth == nil || client.Auth.Extra == nil {
			continue
		}
		if gconv.Uint64(client.Auth.Extra["deviceId"]) == deviceID {
			matched = append(matched, client)
		}
	}
	return matched
}
