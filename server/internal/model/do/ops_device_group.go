package do

import (
	"github.com/gogf/gf/v2/frame/g"
	"github.com/gogf/gf/v2/os/gtime"
)

// OpsDeviceGroup is the golang structure of table hg_ops_device_group for DAO operations like Where/Data.
type OpsDeviceGroup struct {
	g.Meta    `orm:"table:hg_ops_device_group, do:true"`
	Id        any
	Name      any
	Sort      any
	Remark    any
	Status    any
	CreatedAt *gtime.Time
	UpdatedAt *gtime.Time
	DeletedAt *gtime.Time
}
