package common

import (
	"context"
	apicommon "hotgo/api/admin/common"
	"hotgo/internal/service"
)

var Client = cClient{}

type cClient struct{}

func (c *cClient) Register(ctx context.Context, req *apicommon.ClientRegisterReq) (res *apicommon.ClientRegisterRes, err error) {
	data, err := service.SysOpsDevice().ClientRegister(ctx, &req.OpsDeviceClientRegisterInp)
	if err != nil {
		return nil, err
	}

	res = new(apicommon.ClientRegisterRes)
	res.OpsDeviceClientRegisterModel = data
	return
}

func (c *cClient) Heartbeat(ctx context.Context, req *apicommon.ClientHeartbeatReq) (res *apicommon.ClientHeartbeatRes, err error) {
	data, err := service.SysOpsDevice().ClientHeartbeat(ctx, &req.OpsDeviceClientHeartbeatInp)
	if err != nil {
		return nil, err
	}

	res = new(apicommon.ClientHeartbeatRes)
	res.OpsDeviceClientHeartbeatModel = data
	return
}
