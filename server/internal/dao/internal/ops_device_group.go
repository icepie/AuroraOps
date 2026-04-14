package internal

import (
	"context"

	"github.com/gogf/gf/v2/database/gdb"
	"github.com/gogf/gf/v2/frame/g"
)

// OpsDeviceGroupDao is the data access object for the table hg_ops_device_group.
type OpsDeviceGroupDao struct {
	table    string
	group    string
	columns  OpsDeviceGroupColumns
	handlers []gdb.ModelHandler
}

// OpsDeviceGroupColumns defines and stores column names for the table hg_ops_device_group.
type OpsDeviceGroupColumns struct {
	Id        string
	Name      string
	Sort      string
	Remark    string
	Status    string
	CreatedAt string
	UpdatedAt string
	DeletedAt string
}

var opsDeviceGroupColumns = OpsDeviceGroupColumns{
	Id:        "id",
	Name:      "name",
	Sort:      "sort",
	Remark:    "remark",
	Status:    "status",
	CreatedAt: "created_at",
	UpdatedAt: "updated_at",
	DeletedAt: "deleted_at",
}

// NewOpsDeviceGroupDao creates and returns a new DAO object for table data access.
func NewOpsDeviceGroupDao(handlers ...gdb.ModelHandler) *OpsDeviceGroupDao {
	return &OpsDeviceGroupDao{
		group:    "default",
		table:    "hg_ops_device_group",
		columns:  opsDeviceGroupColumns,
		handlers: handlers,
	}
}

func (dao *OpsDeviceGroupDao) DB() gdb.DB {
	return g.DB(dao.group)
}

func (dao *OpsDeviceGroupDao) Table() string {
	return dao.table
}

func (dao *OpsDeviceGroupDao) Columns() OpsDeviceGroupColumns {
	return dao.columns
}

func (dao *OpsDeviceGroupDao) Group() string {
	return dao.group
}

func (dao *OpsDeviceGroupDao) Ctx(ctx context.Context) *gdb.Model {
	model := dao.DB().Model(dao.table)
	for _, handler := range dao.handlers {
		model = handler(model)
	}
	return model.Safe().Ctx(ctx)
}

func (dao *OpsDeviceGroupDao) Transaction(ctx context.Context, f func(ctx context.Context, tx gdb.TX) error) (err error) {
	return dao.Ctx(ctx).Transaction(ctx, f)
}
