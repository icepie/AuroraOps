package opsdevicegroup

import (
	"hotgo/internal/model"
	"hotgo/internal/model/input/sysin"

	"github.com/gogf/gf/v2/frame/g"
)

type ListReq struct {
	g.Meta `path:"/opsDeviceGroup/list" method:"get" tags:"设备分组" summary:"获取设备分组列表"`
}

type ListRes struct {
	List []*sysin.OpsDeviceGroupListModel `json:"list" dc:"数据列表"`
}

type ViewReq struct {
	g.Meta `path:"/opsDeviceGroup/view" method:"get" tags:"设备分组" summary:"获取设备分组详情"`
	sysin.OpsDeviceGroupViewInp
}

type ViewRes struct {
	*sysin.OpsDeviceGroupViewModel
}

type EditReq struct {
	g.Meta `path:"/opsDeviceGroup/edit" method:"post" tags:"设备分组" summary:"修改/新增设备分组"`
	sysin.OpsDeviceGroupEditInp
}

type EditRes struct{}

type DeleteReq struct {
	g.Meta `path:"/opsDeviceGroup/delete" method:"post" tags:"设备分组" summary:"删除设备分组"`
	sysin.OpsDeviceGroupDeleteInp
}

type DeleteRes struct{}

type MaxSortReq struct {
	g.Meta `path:"/opsDeviceGroup/maxSort" method:"get" tags:"设备分组" summary:"获取设备分组最大排序"`
	sysin.OpsDeviceGroupMaxSortInp
}

type MaxSortRes struct {
	*sysin.OpsDeviceGroupMaxSortModel
}

type StatusReq struct {
	g.Meta `path:"/opsDeviceGroup/status" method:"post" tags:"设备分组" summary:"更新设备分组状态"`
	sysin.OpsDeviceGroupStatusInp
}

type StatusRes struct{}

type OptionReq struct {
	g.Meta `path:"/opsDeviceGroup/option" method:"get" tags:"设备分组" summary:"获取设备分组选项"`
}

type OptionRes []*model.Option
