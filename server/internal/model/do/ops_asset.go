// =================================================================================
// Code generated and maintained by GoFrame CLI tool. DO NOT EDIT.
// =================================================================================

package do

import (
	"github.com/gogf/gf/v2/frame/g"
	"github.com/gogf/gf/v2/os/gtime"
)

// OpsAsset is the golang structure of table hg_ops_asset for DAO operations like Where/Data.
type OpsAsset struct {
	g.Meta        `orm:"table:hg_ops_asset, do:true"`
	Id            any         // 资产ID
	DeviceId      any         // 所属设备ID
	AssetType     any         // 资产类型
	AssetName     any         // 资产名称
	Brand         any         // 品牌
	Model         any         // 型号
	SerialNo      any         // 序列号
	Specification any         // 规格参数
	Sort          any         // 排序
	Remark        any         // 备注
	Status        any         // 状态，1正常，2停用
	CreatedAt     *gtime.Time // 创建时间
	UpdatedAt     *gtime.Time // 更新时间
	DeletedAt     *gtime.Time // 删除时间
}
