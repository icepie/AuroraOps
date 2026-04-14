package sys

import (
	"context"
	"hotgo/api/admin/opsdevicegroup"
	"hotgo/internal/model/input/sysin"
	"hotgo/internal/service"
)

var (
	OpsDeviceGroup = cOpsDeviceGroup{}
)

type cOpsDeviceGroup struct{}

func (c *cOpsDeviceGroup) List(ctx context.Context, req *opsdevicegroup.ListReq) (res *opsdevicegroup.ListRes, err error) {
	list, err := service.SysOpsDeviceGroup().List(ctx)
	if err != nil {
		return
	}

	if list == nil {
		list = []*sysin.OpsDeviceGroupListModel{}
	}

	res = new(opsdevicegroup.ListRes)
	res.List = list
	return
}

func (c *cOpsDeviceGroup) Edit(ctx context.Context, req *opsdevicegroup.EditReq) (res *opsdevicegroup.EditRes, err error) {
	err = service.SysOpsDeviceGroup().Edit(ctx, &req.OpsDeviceGroupEditInp)
	return
}

func (c *cOpsDeviceGroup) MaxSort(ctx context.Context, req *opsdevicegroup.MaxSortReq) (res *opsdevicegroup.MaxSortRes, err error) {
	data, err := service.SysOpsDeviceGroup().MaxSort(ctx, &req.OpsDeviceGroupMaxSortInp)
	if err != nil {
		return
	}

	res = new(opsdevicegroup.MaxSortRes)
	res.OpsDeviceGroupMaxSortModel = data
	return
}

func (c *cOpsDeviceGroup) View(ctx context.Context, req *opsdevicegroup.ViewReq) (res *opsdevicegroup.ViewRes, err error) {
	data, err := service.SysOpsDeviceGroup().View(ctx, &req.OpsDeviceGroupViewInp)
	if err != nil {
		return
	}

	res = new(opsdevicegroup.ViewRes)
	res.OpsDeviceGroupViewModel = data
	return
}

func (c *cOpsDeviceGroup) Delete(ctx context.Context, req *opsdevicegroup.DeleteReq) (res *opsdevicegroup.DeleteRes, err error) {
	err = service.SysOpsDeviceGroup().Delete(ctx, &req.OpsDeviceGroupDeleteInp)
	return
}

func (c *cOpsDeviceGroup) Status(ctx context.Context, req *opsdevicegroup.StatusReq) (res *opsdevicegroup.StatusRes, err error) {
	err = service.SysOpsDeviceGroup().Status(ctx, &req.OpsDeviceGroupStatusInp)
	return
}

func (c *cOpsDeviceGroup) Option(ctx context.Context, req *opsdevicegroup.OptionReq) (res opsdevicegroup.OptionRes, err error) {
	data, err := service.SysOpsDeviceGroup().Option(ctx)
	if err != nil {
		return nil, err
	}
	return opsdevicegroup.OptionRes(data), nil
}
