// =================================================================================
// Code generated and maintained by GoFrame CLI tool. DO NOT EDIT.
// =================================================================================

package entity

import (
	"github.com/gogf/gf/v2/os/gtime"
)

// OpsDevice is the golang structure for table ops_device.
type OpsDevice struct {
	Id         uint64      `json:"id"         orm:"id"          description:"设备ID"`
	GroupId    uint64      `json:"groupId"    orm:"group_id"    description:"分组ID"`
	Name       string      `json:"name"       orm:"name"        description:"设备名称"`
	Hostname   string      `json:"hostname"   orm:"hostname"    description:"主机名"`
	Ip         string      `json:"ip"         orm:"ip"          description:"IP地址"`
	DeviceType string      `json:"deviceType" orm:"device_type" description:"设备类型"`
	OsName     string      `json:"osName"     orm:"os_name"     description:"操作系统"`
	Location   string      `json:"location"   orm:"location"    description:"部署位置"`
	Sort       int         `json:"sort"       orm:"sort"        description:"排序"`
	Remark     string      `json:"remark"     orm:"remark"      description:"备注"`
	Status     int         `json:"status"     orm:"status"      description:"状态，1正常，2停用"`
	CreatedAt  *gtime.Time `json:"createdAt"  orm:"created_at"  description:"创建时间"`
	UpdatedAt  *gtime.Time `json:"updatedAt"  orm:"updated_at"  description:"更新时间"`
	DeletedAt  *gtime.Time `json:"deletedAt"  orm:"deleted_at"  description:"删除时间"`
}
