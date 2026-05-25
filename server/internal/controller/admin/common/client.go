package common

import (
	apicommon "auroraops/api/admin/common"
	"auroraops/internal/service"
	"context"
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

func (c *cClient) AssetSync(ctx context.Context, req *apicommon.ClientAssetSyncReq) (res *apicommon.ClientAssetSyncRes, err error) {
	data, err := service.SysOpsAsset().ClientSync(ctx, &req.OpsAssetClientSyncInp)
	if err != nil {
		return nil, err
	}

	res = new(apicommon.ClientAssetSyncRes)
	res.OpsAssetClientSyncModel = data
	return
}

func (c *cClient) AssetPull(ctx context.Context, req *apicommon.ClientAssetPullReq) (res *apicommon.ClientAssetPullRes, err error) {
	data, err := service.SysOpsAsset().ClientPull(ctx, &req.OpsAssetClientPullInp)
	if err != nil {
		return nil, err
	}

	res = new(apicommon.ClientAssetPullRes)
	res.OpsAssetClientPullModel = data
	return
}
