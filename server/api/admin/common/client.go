package common

import (
	"hotgo/internal/model/input/sysin"

	"github.com/gogf/gf/v2/frame/g"
)

type ClientRegisterReq struct {
	g.Meta `path:"/client/register" method:"post" tags:"客户端接入" summary:"客户端检查连接并自动注册设备"`
	sysin.OpsDeviceClientRegisterInp
}

type ClientRegisterRes struct {
	*sysin.OpsDeviceClientRegisterModel
}

type ClientHeartbeatReq struct {
	g.Meta `path:"/client/heartbeat" method:"post" tags:"客户端接入" summary:"客户端设备心跳"`
	sysin.OpsDeviceClientHeartbeatInp
}

type ClientHeartbeatRes struct {
	*sysin.OpsDeviceClientHeartbeatModel
}
