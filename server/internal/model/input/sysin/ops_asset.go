package sysin

import (
	"auroraops/internal/consts"
	"auroraops/internal/model/entity"
	"auroraops/internal/model/input/form"
	"auroraops/utility/validate"
	"context"

	"github.com/gogf/gf/v2/errors/gerror"
	"github.com/gogf/gf/v2/frame/g"
	"github.com/gogf/gf/v2/os/gtime"
)

type OpsAssetUpdateFields struct {
	DeviceId      uint64 `json:"deviceId"      dc:"所属设备"`
	AssetType     string `json:"assetType"     dc:"资产类型"`
	AssetName     string `json:"assetName"     dc:"资产名称"`
	Brand         string `json:"brand"         dc:"品牌"`
	Model         string `json:"model"         dc:"型号"`
	SerialNo      string `json:"serialNo"      dc:"序列号"`
	Specification string `json:"specification" dc:"规格参数"`
	Source        string `json:"source"        dc:"资产来源"`
	SyncHash      string `json:"syncHash"      dc:"同步摘要"`
	Sort          int    `json:"sort"          dc:"排序"`
	Remark        string `json:"remark"        dc:"备注"`
	Status        int    `json:"status"        dc:"状态"`
}

type OpsAssetInsertFields struct {
	DeviceId      uint64 `json:"deviceId"      dc:"所属设备"`
	AssetType     string `json:"assetType"     dc:"资产类型"`
	AssetName     string `json:"assetName"     dc:"资产名称"`
	Brand         string `json:"brand"         dc:"品牌"`
	Model         string `json:"model"         dc:"型号"`
	SerialNo      string `json:"serialNo"      dc:"序列号"`
	Specification string `json:"specification" dc:"规格参数"`
	Source        string `json:"source"        dc:"资产来源"`
	SyncHash      string `json:"syncHash"      dc:"同步摘要"`
	Sort          int    `json:"sort"          dc:"排序"`
	Remark        string `json:"remark"        dc:"备注"`
	Status        int    `json:"status"        dc:"状态"`
}

type OpsAssetEditInp struct {
	entity.OpsAsset
}

func (in *OpsAssetEditInp) Filter(ctx context.Context) (err error) {
	if in.DeviceId == 0 {
		return gerror.New("所属设备不能为空")
	}
	if verr := g.Validator().Rules("required").Data(in.AssetType).Messages("资产类型不能为空").Run(ctx); verr != nil {
		return verr.Current()
	}
	if verr := g.Validator().Rules("required").Data(in.AssetName).Messages("资产名称不能为空").Run(ctx); verr != nil {
		return verr.Current()
	}
	if verr := g.Validator().Rules("required").Data(in.Sort).Messages("排序不能为空").Run(ctx); verr != nil {
		return verr.Current()
	}
	return
}

type OpsAssetEditModel struct{}

type OpsAssetDeleteInp struct {
	Id interface{} `json:"id" v:"required#资产ID不能为空" dc:"资产ID"`
}

func (in *OpsAssetDeleteInp) Filter(ctx context.Context) (err error) {
	return
}

type OpsAssetDeleteModel struct{}

type OpsAssetViewInp struct {
	Id uint64 `json:"id" v:"required#资产ID不能为空" dc:"资产ID"`
}

func (in *OpsAssetViewInp) Filter(ctx context.Context) (err error) {
	return
}

type OpsAssetViewModel struct {
	entity.OpsAsset
}

type OpsAssetListInp struct {
	form.PageReq
	Id        uint64        `json:"id"        dc:"资产ID"`
	DeviceId  uint64        `json:"deviceId"  dc:"所属设备"`
	AssetType string        `json:"assetType" dc:"资产类型"`
	AssetName string        `json:"assetName" dc:"资产名称"`
	Status    int           `json:"status"    dc:"状态"`
	CreatedAt []*gtime.Time `json:"createdAt" dc:"创建时间"`
}

func (in *OpsAssetListInp) Filter(ctx context.Context) (err error) {
	return
}

type OpsAssetListModel struct {
	Id         uint64      `json:"id"         dc:"资产ID"`
	DeviceId   uint64      `json:"deviceId"   dc:"所属设备ID"`
	DeviceName string      `json:"deviceName" dc:"所属设备"`
	AssetType  string      `json:"assetType"  dc:"资产类型"`
	AssetName  string      `json:"assetName"  dc:"资产名称"`
	Brand      string      `json:"brand"      dc:"品牌"`
	Model      string      `json:"model"      dc:"型号"`
	SerialNo   string      `json:"serialNo"   dc:"序列号"`
	Source     string      `json:"source"     dc:"资产来源"`
	LastSeenAt *gtime.Time `json:"lastSeenAt" dc:"最近观测时间"`
	Status     int         `json:"status"     dc:"状态"`
	CreatedAt  *gtime.Time `json:"createdAt"  dc:"创建时间"`
}

type OpsAssetMaxSortInp struct{}

func (in *OpsAssetMaxSortInp) Filter(ctx context.Context) (err error) {
	return
}

type OpsAssetMaxSortModel struct {
	Sort int `json:"sort" dc:"排序"`
}

type OpsAssetStatusInp struct {
	Id     uint64 `json:"id"     v:"required#资产ID不能为空" dc:"资产ID"`
	Status int    `json:"status" dc:"状态"`
}

func (in *OpsAssetStatusInp) Filter(ctx context.Context) (err error) {
	if in.Id == 0 {
		return gerror.New("资产ID不能为空")
	}
	if in.Status <= 0 {
		return gerror.New("状态不能为空")
	}
	if !validate.InSlice(consts.StatusSlice, in.Status) {
		return gerror.New("状态不正确")
	}
	return
}

type OpsAssetStatusModel struct{}

type OpsAssetSyncItem struct {
	AssetType     string `json:"assetType"     dc:"资产类型"`
	UniqueKey     string `json:"uniqueKey"     dc:"资产唯一键"`
	AssetName     string `json:"assetName"     dc:"资产名称"`
	Brand         string `json:"brand"         dc:"品牌"`
	Model         string `json:"model"         dc:"型号"`
	SerialNo      string `json:"serialNo"      dc:"序列号"`
	Specification string `json:"specification" dc:"规格参数"`
	Source        string `json:"source"        dc:"资产来源"`
	SyncHash      string `json:"syncHash"      dc:"同步摘要"`
	Remark        string `json:"remark"        dc:"备注"`
}

type OpsAssetClientSyncInp struct {
	DeviceId uint64              `json:"deviceId" dc:"设备ID"`
	Assets   []*OpsAssetSyncItem `json:"assets"   dc:"资产列表"`
}

func (in *OpsAssetClientSyncInp) Filter(ctx context.Context) (err error) {
	if in.DeviceId == 0 {
		return gerror.New("设备ID不能为空")
	}
	if len(in.Assets) == 0 {
		return gerror.New("资产列表不能为空")
	}
	for _, item := range in.Assets {
		if item == nil {
			return gerror.New("资产项不能为空")
		}
		if item.AssetType == "" {
			return gerror.New("资产类型不能为空")
		}
		if item.UniqueKey == "" {
			return gerror.New("资产唯一键不能为空")
		}
		if item.AssetName == "" {
			return gerror.New("资产名称不能为空")
		}
	}
	return
}

type OpsAssetClientSyncModel struct {
	DeviceId uint64 `json:"deviceId" dc:"设备ID"`
	Total    int    `json:"total"    dc:"本次同步资产数"`
	Created  int    `json:"created"  dc:"新增资产数"`
	Updated  int    `json:"updated"  dc:"更新资产数"`
	Disabled int    `json:"disabled" dc:"停用资产数"`
}

type OpsAssetClientPullInp struct {
	DeviceId uint64 `json:"deviceId" dc:"设备ID"`
}

func (in *OpsAssetClientPullInp) Filter(ctx context.Context) (err error) {
	if in.DeviceId == 0 {
		return gerror.New("设备ID不能为空")
	}
	return nil
}

type OpsAssetClientPullItem struct {
	Id            uint64      `json:"id"            dc:"资产ID"`
	AssetType     string      `json:"assetType"     dc:"资产类型"`
	UniqueKey     string      `json:"uniqueKey"     dc:"资产唯一键"`
	AssetName     string      `json:"assetName"     dc:"资产名称"`
	Brand         string      `json:"brand"         dc:"品牌"`
	Model         string      `json:"model"         dc:"型号"`
	SerialNo      string      `json:"serialNo"      dc:"序列号"`
	Specification string      `json:"specification" dc:"规格参数"`
	Source        string      `json:"source"        dc:"资产来源"`
	SyncHash      string      `json:"syncHash"      dc:"同步摘要"`
	Remark        string      `json:"remark"        dc:"备注"`
	Status        int         `json:"status"        dc:"状态"`
	LastSeenAt    *gtime.Time `json:"lastSeenAt"    dc:"最近观测时间"`
}

type OpsAssetClientPullModel struct {
	DeviceId uint64                    `json:"deviceId" dc:"设备ID"`
	Assets   []*OpsAssetClientPullItem `json:"assets"   dc:"资产列表"`
}
