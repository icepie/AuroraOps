package opshardware

import (
	"auroraops/internal/model/input/form"
	"auroraops/internal/model/input/sysin"
	"github.com/gogf/gf/v2/frame/g"
)

type OverviewReq struct {
	g.Meta `path:"/opsHardware/overview" method:"get" tags:"硬件管理" summary:"获取硬件管理概览列表"`
	sysin.OpsHardwareOverviewInp
}

type OverviewRes struct {
	form.PageRes
	List []*sysin.OpsHardwareOverviewModel `json:"list" dc:"数据列表"`
}

type ExportReq struct {
	g.Meta `path:"/opsHardware/export" method:"get" tags:"硬件管理" summary:"导出硬件管理概览列表"`
	sysin.OpsHardwareOverviewInp
}

type ExportRes struct{}
