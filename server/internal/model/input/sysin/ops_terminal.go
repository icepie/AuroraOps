package sysin

import (
	"context"

	"github.com/gogf/gf/v2/errors/gerror"
)

type OpsDeviceTerminalCreateInp struct {
	DeviceId uint64 `json:"deviceId" dc:"设备ID"`
}

func (in *OpsDeviceTerminalCreateInp) Filter(ctx context.Context) (err error) {
	if in.DeviceId == 0 {
		return gerror.New("设备ID不能为空")
	}
	return nil
}

type OpsDeviceTerminalCreateModel struct {
	SessionId string `json:"sessionId" dc:"终端会话ID"`
	WsPath    string `json:"wsPath"    dc:"终端WebSocket路径"`
	PagePath  string `json:"pagePath"  dc:"终端页面路径"`
}
