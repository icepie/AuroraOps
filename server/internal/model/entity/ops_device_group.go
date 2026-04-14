package entity

import (
	"github.com/gogf/gf/v2/os/gtime"
)

// OpsDeviceGroup is the golang structure for table hg_ops_device_group.
type OpsDeviceGroup struct {
	Id        uint64      `json:"id"        orm:"id"         description:"分组ID"`
	Name      string      `json:"name"      orm:"name"       description:"分组名称"`
	Sort      int         `json:"sort"      orm:"sort"       description:"排序"`
	Remark    string      `json:"remark"    orm:"remark"     description:"备注"`
	Status    int         `json:"status"    orm:"status"     description:"状态，1正常，2停用"`
	CreatedAt *gtime.Time `json:"createdAt" orm:"created_at" description:"创建时间"`
	UpdatedAt *gtime.Time `json:"updatedAt" orm:"updated_at" description:"更新时间"`
	DeletedAt *gtime.Time `json:"deletedAt" orm:"deleted_at" description:"删除时间"`
}
