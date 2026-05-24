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

type OpsDeviceDesktopCreateInp struct {
	DeviceId uint64 `json:"deviceId" dc:"设备ID"`
}

func (in *OpsDeviceDesktopCreateInp) Filter(ctx context.Context) (err error) {
	if in.DeviceId == 0 {
		return gerror.New("设备ID不能为空")
	}
	return nil
}

type OpsDeviceDesktopCreateModel struct {
	SessionId  string `json:"sessionId"  dc:"桌面会话ID"`
	WsPath     string `json:"wsPath"     dc:"桌面WebSocket路径"`
	PagePath   string `json:"pagePath"   dc:"桌面页面路径"`
	WeylusPath string `json:"weylusPath" dc:"Weylus原生页面反代路径"`
}
