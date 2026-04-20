package opshardware

import (
	"github.com/gogf/gf/v2/frame/g"
	"hotgo/internal/model/input/form"
	"hotgo/internal/model/input/sysin"
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
