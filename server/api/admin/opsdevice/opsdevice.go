package opsdevice

import (
	"hotgo/internal/model"
	"hotgo/internal/model/input/form"
	"hotgo/internal/model/input/sysin"

	"github.com/gogf/gf/v2/frame/g"
)

type ListReq struct {
	g.Meta `path:"/opsDevice/list" method:"get" tags:"运维设备" summary:"获取运维设备列表"`
	sysin.OpsDeviceListInp
}

type ListRes struct {
	form.PageRes
	List []*sysin.OpsDeviceListModel `json:"list" dc:"数据列表"`
}

type ViewReq struct {
	g.Meta `path:"/opsDevice/view" method:"get" tags:"运维设备" summary:"获取运维设备详情"`
	sysin.OpsDeviceViewInp
}

type ViewRes struct {
	*sysin.OpsDeviceViewModel
}

type EditReq struct {
	g.Meta `path:"/opsDevice/edit" method:"post" tags:"运维设备" summary:"修改/新增运维设备"`
	sysin.OpsDeviceEditInp
}

type EditRes struct{}

type DeleteReq struct {
	g.Meta `path:"/opsDevice/delete" method:"post" tags:"运维设备" summary:"删除运维设备"`
	sysin.OpsDeviceDeleteInp
}

type DeleteRes struct{}

type MaxSortReq struct {
	g.Meta `path:"/opsDevice/maxSort" method:"get" tags:"运维设备" summary:"获取运维设备最大排序"`
	sysin.OpsDeviceMaxSortInp
}

type MaxSortRes struct {
	*sysin.OpsDeviceMaxSortModel
}

type StatusReq struct {
	g.Meta `path:"/opsDevice/status" method:"post" tags:"运维设备" summary:"更新运维设备状态"`
	sysin.OpsDeviceStatusInp
}

type StatusRes struct{}

type OptionReq struct {
	g.Meta `path:"/opsDevice/option" method:"get" tags:"运维设备" summary:"获取运维设备选项"`
}

type OptionRes []*model.Option

type TerminalCreateReq struct {
	g.Meta `path:"/opsDevice/terminal/create" method:"post" tags:"运维设备" summary:"创建设备远程终端会话"`
	sysin.OpsDeviceTerminalCreateInp
}

type TerminalCreateRes struct {
	*sysin.OpsDeviceTerminalCreateModel
}

type DesktopCreateReq struct {
	g.Meta `path:"/opsDevice/desktop/create" method:"post" tags:"运维设备" summary:"创建设备远程桌面会话"`
	sysin.OpsDeviceDesktopCreateInp
}

type DesktopCreateRes struct {
	*sysin.OpsDeviceDesktopCreateModel
}
