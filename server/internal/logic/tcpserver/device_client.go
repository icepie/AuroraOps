package tcpserver

import (
	"fmt"

	"github.com/gogf/gf/v2/util/gconv"

	"hotgo/internal/library/network/tcp"
)

func (s *sTCPServer) getDeviceClients(deviceID uint64) []*tcp.Conn {
	if deviceID == 0 || s.serv == nil {
		return nil
	}
	clients := s.serv.GetAppIdClients(fmt.Sprintf("device:%d", deviceID))
	if len(clients) > 0 {
		return clients
	}

	groupClients := s.serv.GetGroupClients("device")
	matched := make([]*tcp.Conn, 0, len(groupClients))
	for _, client := range groupClients {
		if client == nil || client.Auth == nil || client.Auth.Extra == nil {
			continue
		}
		if gconv.Uint64(client.Auth.Extra["deviceId"]) == deviceID {
			matched = append(matched, client)
		}
	}
	return matched
}
