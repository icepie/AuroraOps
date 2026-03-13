package sys

import (
	"context"
	"hotgo/api/admin/opsdevice"
	"hotgo/internal/model/input/sysin"
	"hotgo/internal/service"
)

var (
	OpsDevice = cOpsDevice{}
)

type cOpsDevice struct{}

func (c *cOpsDevice) List(ctx context.Context, req *opsdevice.ListReq) (res *opsdevice.ListRes, err error) {
	list, totalCount, err := service.SysOpsDevice().List(ctx, &req.OpsDeviceListInp)
	if err != nil {
		return
	}

	if list == nil {
		list = []*sysin.OpsDeviceListModel{}
	}

	res = new(opsdevice.ListRes)
	res.List = list
	res.PageRes.Pack(req, totalCount)
	return
}

func (c *cOpsDevice) Edit(ctx context.Context, req *opsdevice.EditReq) (res *opsdevice.EditRes, err error) {
	err = service.SysOpsDevice().Edit(ctx, &req.OpsDeviceEditInp)
	return
}

func (c *cOpsDevice) MaxSort(ctx context.Context, req *opsdevice.MaxSortReq) (res *opsdevice.MaxSortRes, err error) {
	data, err := service.SysOpsDevice().MaxSort(ctx, &req.OpsDeviceMaxSortInp)
	if err != nil {
		return
	}

	res = new(opsdevice.MaxSortRes)
	res.OpsDeviceMaxSortModel = data
	return
}

func (c *cOpsDevice) View(ctx context.Context, req *opsdevice.ViewReq) (res *opsdevice.ViewRes, err error) {
	data, err := service.SysOpsDevice().View(ctx, &req.OpsDeviceViewInp)
	if err != nil {
		return
	}

	res = new(opsdevice.ViewRes)
	res.OpsDeviceViewModel = data
	return
}

func (c *cOpsDevice) Delete(ctx context.Context, req *opsdevice.DeleteReq) (res *opsdevice.DeleteRes, err error) {
	err = service.SysOpsDevice().Delete(ctx, &req.OpsDeviceDeleteInp)
	return
}

func (c *cOpsDevice) Status(ctx context.Context, req *opsdevice.StatusReq) (res *opsdevice.StatusRes, err error) {
	err = service.SysOpsDevice().Status(ctx, &req.OpsDeviceStatusInp)
	return
}

func (c *cOpsDevice) Option(ctx context.Context, req *opsdevice.OptionReq) (res opsdevice.OptionRes, err error) {
	data, err := service.SysOpsDevice().Option(ctx)
	if err != nil {
		return nil, err
	}
	return opsdevice.OptionRes(data), nil
}
