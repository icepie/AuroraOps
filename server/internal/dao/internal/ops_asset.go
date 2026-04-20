// ==========================================================================
// Code generated and maintained by GoFrame CLI tool. DO NOT EDIT.
// ==========================================================================

package internal

import (
	"context"

	"github.com/gogf/gf/v2/database/gdb"
	"github.com/gogf/gf/v2/frame/g"
)

// OpsAssetDao is the data access object for the table hg_ops_asset.
type OpsAssetDao struct {
	table    string             // table is the underlying table name of the DAO.
	group    string             // group is the database configuration group name of the current DAO.
	columns  OpsAssetColumns    // columns contains all the column names of Table for convenient usage.
	handlers []gdb.ModelHandler // handlers for customized model modification.
}

// OpsAssetColumns defines and stores column names for the table hg_ops_asset.
type OpsAssetColumns struct {
	Id            string // 资产ID
	DeviceId      string // 所属设备ID
	AssetType     string // 资产类型
	AssetName     string // 资产名称
	Brand         string // 品牌
	Model         string // 型号
	SerialNo      string // 序列号
	Specification string // 规格参数
	Source        string // 资产来源
	SyncHash      string // 同步摘要
	LastSeenAt    string // 最近观测时间
	Sort          string // 排序
	Remark        string // 备注
	Status        string // 状态，1正常，2停用
	CreatedAt     string // 创建时间
	UpdatedAt     string // 更新时间
	DeletedAt     string // 删除时间
}

// opsAssetColumns holds the columns for the table hg_ops_asset.
var opsAssetColumns = OpsAssetColumns{
	Id:            "id",
	DeviceId:      "device_id",
	AssetType:     "asset_type",
	AssetName:     "asset_name",
	Brand:         "brand",
	Model:         "model",
	SerialNo:      "serial_no",
	Specification: "specification",
	Source:        "source",
	SyncHash:      "sync_hash",
	LastSeenAt:    "last_seen_at",
	Sort:          "sort",
	Remark:        "remark",
	Status:        "status",
	CreatedAt:     "created_at",
	UpdatedAt:     "updated_at",
	DeletedAt:     "deleted_at",
}

// NewOpsAssetDao creates and returns a new DAO object for table data access.
func NewOpsAssetDao(handlers ...gdb.ModelHandler) *OpsAssetDao {
	return &OpsAssetDao{
		group:    "default",
		table:    "hg_ops_asset",
		columns:  opsAssetColumns,
		handlers: handlers,
	}
}

// DB retrieves and returns the underlying raw database management object of the current DAO.
func (dao *OpsAssetDao) DB() gdb.DB {
	return g.DB(dao.group)
}

// Table returns the table name of the current DAO.
func (dao *OpsAssetDao) Table() string {
	return dao.table
}

// Columns returns all column names of the current DAO.
func (dao *OpsAssetDao) Columns() OpsAssetColumns {
	return dao.columns
}

// Group returns the database configuration group name of the current DAO.
func (dao *OpsAssetDao) Group() string {
	return dao.group
}

// Ctx creates and returns a Model for the current DAO. It automatically sets the context for the current operation.
func (dao *OpsAssetDao) Ctx(ctx context.Context) *gdb.Model {
	model := dao.DB().Model(dao.table)
	for _, handler := range dao.handlers {
		model = handler(model)
	}
	return model.Safe().Ctx(ctx)
}

// Transaction wraps the transaction logic using function f.
// It rolls back the transaction and returns the error if function f returns a non-nil error.
// It commits the transaction and returns nil if function f returns nil.
//
// Note: Do not commit or roll back the transaction in function f,
// as it is automatically handled by this function.
func (dao *OpsAssetDao) Transaction(ctx context.Context, f func(ctx context.Context, tx gdb.TX) error) (err error) {
	return dao.Ctx(ctx).Transaction(ctx, f)
}
