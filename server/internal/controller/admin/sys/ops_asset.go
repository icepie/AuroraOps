package sys

import (
	"auroraops/api/admin/opsasset"
	"auroraops/internal/model/input/sysin"
	"auroraops/internal/service"
	"context"
)

var (
	OpsAsset = cOpsAsset{}
)

type cOpsAsset struct{}

func (c *cOpsAsset) List(ctx context.Context, req *opsasset.ListReq) (res *opsasset.ListRes, err error) {
	list, totalCount, err := service.SysOpsAsset().List(ctx, &req.OpsAssetListInp)
	if err != nil {
		return
	}

	if list == nil {
		list = []*sysin.OpsAssetListModel{}
	}

	res = new(opsasset.ListRes)
	res.List = list
	res.PageRes.Pack(req, totalCount)
	return
}

func (c *cOpsAsset) Edit(ctx context.Context, req *opsasset.EditReq) (res *opsasset.EditRes, err error) {
	err = service.SysOpsAsset().Edit(ctx, &req.OpsAssetEditInp)
	return
}

func (c *cOpsAsset) MaxSort(ctx context.Context, req *opsasset.MaxSortReq) (res *opsasset.MaxSortRes, err error) {
	data, err := service.SysOpsAsset().MaxSort(ctx, &req.OpsAssetMaxSortInp)
	if err != nil {
		return
	}

	res = new(opsasset.MaxSortRes)
	res.OpsAssetMaxSortModel = data
	return
}

func (c *cOpsAsset) View(ctx context.Context, req *opsasset.ViewReq) (res *opsasset.ViewRes, err error) {
	data, err := service.SysOpsAsset().View(ctx, &req.OpsAssetViewInp)
	if err != nil {
		return
	}

	res = new(opsasset.ViewRes)
	res.OpsAssetViewModel = data
	return
}

func (c *cOpsAsset) Delete(ctx context.Context, req *opsasset.DeleteReq) (res *opsasset.DeleteRes, err error) {
	err = service.SysOpsAsset().Delete(ctx, &req.OpsAssetDeleteInp)
	return
}

func (c *cOpsAsset) Status(ctx context.Context, req *opsasset.StatusReq) (res *opsasset.StatusRes, err error) {
	err = service.SysOpsAsset().Status(ctx, &req.OpsAssetStatusInp)
	return
}
