package sys

import (
	"auroraops/api/admin/opshardware"
	"auroraops/internal/model/input/sysin"
	"auroraops/internal/service"
	"context"
)

var (
	OpsHardware = cOpsHardware{}
)

type cOpsHardware struct{}

func (c *cOpsHardware) Overview(ctx context.Context, req *opshardware.OverviewReq) (res *opshardware.OverviewRes, err error) {
	list, totalCount, err := service.SysOpsHardware().Overview(ctx, &req.OpsHardwareOverviewInp)
	if err != nil {
		return
	}

	if list == nil {
		list = []*sysin.OpsHardwareOverviewModel{}
	}

	res = new(opshardware.OverviewRes)
	res.List = list
	res.PageRes.Pack(req, totalCount)
	return
}

func (c *cOpsHardware) Export(ctx context.Context, req *opshardware.ExportReq) (res *opshardware.ExportRes, err error) {
	err = service.SysOpsHardware().Export(ctx, &req.OpsHardwareOverviewInp)
	return
}
