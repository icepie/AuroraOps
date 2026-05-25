package opsasset

import (
	"auroraops/internal/model/input/form"
	"auroraops/internal/model/input/sysin"

	"github.com/gogf/gf/v2/frame/g"
)

type ListReq struct {
	g.Meta `path:"/opsAsset/list" method:"get" tags:"运维资产" summary:"获取运维资产列表"`
	sysin.OpsAssetListInp
}

type ListRes struct {
	form.PageRes
	List []*sysin.OpsAssetListModel `json:"list" dc:"数据列表"`
}

type ViewReq struct {
	g.Meta `path:"/opsAsset/view" method:"get" tags:"运维资产" summary:"获取运维资产详情"`
	sysin.OpsAssetViewInp
}

type ViewRes struct {
	*sysin.OpsAssetViewModel
}

type EditReq struct {
	g.Meta `path:"/opsAsset/edit" method:"post" tags:"运维资产" summary:"修改/新增运维资产"`
	sysin.OpsAssetEditInp
}

type EditRes struct{}

type DeleteReq struct {
	g.Meta `path:"/opsAsset/delete" method:"post" tags:"运维资产" summary:"删除运维资产"`
	sysin.OpsAssetDeleteInp
}

type DeleteRes struct{}

type MaxSortReq struct {
	g.Meta `path:"/opsAsset/maxSort" method:"get" tags:"运维资产" summary:"获取运维资产最大排序"`
	sysin.OpsAssetMaxSortInp
}

type MaxSortRes struct {
	*sysin.OpsAssetMaxSortModel
}

type StatusReq struct {
	g.Meta `path:"/opsAsset/status" method:"post" tags:"运维资产" summary:"更新运维资产状态"`
	sysin.OpsAssetStatusInp
}

type StatusRes struct{}
