package dao

import (
	"hotgo/internal/dao/internal"
)

// opsDeviceGroupDao is the data access object for the table hg_ops_device_group.
type opsDeviceGroupDao struct {
	*internal.OpsDeviceGroupDao
}

var (
	// OpsDeviceGroup is a globally accessible object for table hg_ops_device_group operations.
	OpsDeviceGroup = opsDeviceGroupDao{internal.NewOpsDeviceGroupDao()}
)
