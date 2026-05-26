// =================================================================================
// Code generated and maintained by GoFrame CLI tool. DO NOT EDIT.
// =================================================================================

package entity

import (
	"github.com/gogf/gf/v2/os/gtime"
)

// OpsAsset is the golang structure for table ops_asset.
type OpsAsset struct {
	Id            uint64      `json:"id"            orm:"id"            description:"资产ID"`
	DeviceId      uint64      `json:"deviceId"      orm:"device_id"     description:"所属设备ID"`
	AssetType     string      `json:"assetType"     orm:"asset_type"    description:"资产类型"`
	UniqueKey     string      `json:"uniqueKey"     orm:"unique_key"    description:"资产唯一键"`
	AssetName     string      `json:"assetName"     orm:"asset_name"    description:"资产名称"`
	Brand         string      `json:"brand"         orm:"brand"         description:"品牌"`
	Model         string      `json:"model"         orm:"model"         description:"型号"`
	SerialNo      string      `json:"serialNo"      orm:"serial_no"     description:"序列号"`
	Specification string      `json:"specification" orm:"specification" description:"规格参数"`
	Source        string      `json:"source"        orm:"source"        description:"资产来源"`
	SyncHash      string      `json:"syncHash"      orm:"sync_hash"     description:"同步摘要"`
	LastSeenAt    *gtime.Time `json:"lastSeenAt"    orm:"last_seen_at"  description:"最近观测时间"`
	Sort          int         `json:"sort"          orm:"sort"          description:"排序"`
	Remark        string      `json:"remark"        orm:"remark"        description:"备注"`
	Status        int         `json:"status"        orm:"status"        description:"状态，1正常，2停用"`
	CreatedAt     *gtime.Time `json:"createdAt"     orm:"created_at"    description:"创建时间"`
	UpdatedAt     *gtime.Time `json:"updatedAt"     orm:"updated_at"    description:"更新时间"`
	DeletedAt     *gtime.Time `json:"deletedAt"     orm:"deleted_at"    description:"删除时间"`
}
