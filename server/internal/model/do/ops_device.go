// =================================================================================
// Code generated and maintained by GoFrame CLI tool. DO NOT EDIT.
// =================================================================================

package do

import (
	"github.com/gogf/gf/v2/frame/g"
	"github.com/gogf/gf/v2/os/gtime"
)

// OpsDevice is the golang structure of table hg_ops_device for DAO operations like Where/Data.
type OpsDevice struct {
	g.Meta            `orm:"table:hg_ops_device, do:true"`
	Id                any         // 设备ID
	GroupId           any         // 分组ID
	Name              any         // 设备名称
	Hostname          any         // 主机名
	Ip                any         // IP地址
	MacAddress        any         // MAC地址
	DeviceType        any         // 设备类型
	OsName            any         // 操作系统
	Architecture      any         // 系统架构
	Location          any         // 部署位置
	MonitorSnapshot   any         // 监视快照
	MonitorReportedAt *gtime.Time // 监视上报时间
	Sort              any         // 排序
	Remark            any         // 备注
	Status            any         // 状态，1正常，2停用
	CreatedAt         *gtime.Time // 创建时间
	UpdatedAt         *gtime.Time // 更新时间
	DeletedAt         *gtime.Time // 删除时间
}
