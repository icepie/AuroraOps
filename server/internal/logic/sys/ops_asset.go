package sys

import (
	"context"
	"fmt"
	"hotgo/internal/dao"
	"hotgo/internal/library/hgorm/handler"
	"hotgo/internal/model/do"
	"hotgo/internal/model/input/form"
	"hotgo/internal/model/input/sysin"
	"hotgo/internal/service"

	"github.com/gogf/gf/v2/database/gdb"
	"github.com/gogf/gf/v2/errors/gerror"
	"github.com/gogf/gf/v2/frame/g"
)

type sSysOpsAsset struct{}

func NewSysOpsAsset() *sSysOpsAsset {
	return &sSysOpsAsset{}
}

func init() {
	service.RegisterSysOpsAsset(NewSysOpsAsset())
}

func (s *sSysOpsAsset) Model(ctx context.Context, option ...*handler.Option) *gdb.Model {
	return handler.Model(dao.OpsAsset.Ctx(ctx), option...)
}

func (s *sSysOpsAsset) List(ctx context.Context, in *sysin.OpsAssetListInp) (list []*sysin.OpsAssetListModel, totalCount int, err error) {
	mod := s.Model(ctx).As("a").
		LeftJoin(
			fmt.Sprintf("%s d", dao.OpsDevice.Table()),
			fmt.Sprintf("a.%s = d.%s", dao.OpsAsset.Columns().DeviceId, dao.OpsDevice.Columns().Id),
		).
		Fields(
			"a.id",
			"a.device_id",
			"d.name as device_name",
			"a.asset_type",
			"a.asset_name",
			"a.brand",
			"a.model",
			"a.serial_no",
			"a.status",
			"a.created_at",
		)

	if in.Id > 0 {
		mod = mod.Where("a."+dao.OpsAsset.Columns().Id, in.Id)
	}
	if in.DeviceId > 0 {
		mod = mod.Where("a."+dao.OpsAsset.Columns().DeviceId, in.DeviceId)
	}
	if in.AssetType != "" {
		mod = mod.Where("a."+dao.OpsAsset.Columns().AssetType, in.AssetType)
	}
	if in.AssetName != "" {
		mod = mod.WhereLike("a."+dao.OpsAsset.Columns().AssetName, "%"+in.AssetName+"%")
	}
	if in.Status > 0 {
		mod = mod.Where("a."+dao.OpsAsset.Columns().Status, in.Status)
	}
	if len(in.CreatedAt) == 2 {
		mod = mod.WhereBetween("a."+dao.OpsAsset.Columns().CreatedAt, in.CreatedAt[0], in.CreatedAt[1])
	}

	mod = mod.Page(in.Page, in.PerPage)
	mod = mod.OrderAsc("a." + dao.OpsAsset.Columns().Sort).OrderDesc("a." + dao.OpsAsset.Columns().Id)

	if err = mod.ScanAndCount(&list, &totalCount, false); err != nil {
		return nil, 0, gerror.Wrap(err, "获取运维资产列表失败，请稍后重试！")
	}
	return
}

func (s *sSysOpsAsset) Edit(ctx context.Context, in *sysin.OpsAssetEditInp) (err error) {
	exists, err := dao.OpsDevice.Ctx(ctx).WherePri(in.DeviceId).Count()
	if err != nil {
		return gerror.Wrap(err, "校验所属设备失败，请稍后重试！")
	}
	if exists == 0 {
		return gerror.New("所属设备不存在，请重新选择")
	}

	data := do.OpsAsset{
		DeviceId:      in.DeviceId,
		AssetType:     in.AssetType,
		AssetName:     in.AssetName,
		Brand:         in.Brand,
		Model:         in.Model,
		SerialNo:      in.SerialNo,
		Specification: in.Specification,
		Sort:          in.Sort,
		Remark:        in.Remark,
		Status:        in.Status,
	}

	return g.DB().Transaction(ctx, func(ctx context.Context, tx gdb.TX) (err error) {
		if in.Id > 0 {
			if _, err = s.Model(ctx).
				Fields(sysin.OpsAssetUpdateFields{}).
				WherePri(in.Id).
				Data(data).
				Update(); err != nil {
				return gerror.Wrap(err, "修改运维资产失败，请稍后重试！")
			}
			return nil
		}

		if _, err = s.Model(ctx, &handler.Option{FilterAuth: false}).
			Fields(sysin.OpsAssetInsertFields{}).
			Data(data).
			OmitEmptyData().
			Insert(); err != nil {
			return gerror.Wrap(err, "新增运维资产失败，请稍后重试！")
		}
		return nil
	})
}

func (s *sSysOpsAsset) Delete(ctx context.Context, in *sysin.OpsAssetDeleteInp) (err error) {
	if _, err = s.Model(ctx).WherePri(in.Id).Unscoped().Delete(); err != nil {
		return gerror.Wrap(err, "删除运维资产失败，请稍后重试！")
	}
	return nil
}

func (s *sSysOpsAsset) MaxSort(ctx context.Context, in *sysin.OpsAssetMaxSortInp) (res *sysin.OpsAssetMaxSortModel, err error) {
	if err = dao.OpsAsset.Ctx(ctx).Fields(dao.OpsAsset.Columns().Sort).OrderDesc(dao.OpsAsset.Columns().Sort).Scan(&res); err != nil {
		return nil, gerror.Wrap(err, "获取运维资产最大排序失败，请稍后重试！")
	}
	if res == nil {
		res = new(sysin.OpsAssetMaxSortModel)
	}
	res.Sort = form.DefaultMaxSort(res.Sort)
	return
}

func (s *sSysOpsAsset) View(ctx context.Context, in *sysin.OpsAssetViewInp) (res *sysin.OpsAssetViewModel, err error) {
	if err = s.Model(ctx).WherePri(in.Id).Scan(&res); err != nil {
		return nil, gerror.Wrap(err, "获取运维资产信息失败，请稍后重试！")
	}
	return
}

func (s *sSysOpsAsset) Status(ctx context.Context, in *sysin.OpsAssetStatusInp) (err error) {
	if _, err = s.Model(ctx).
		WherePri(in.Id).
		Data(do.OpsAsset{Status: in.Status}).
		Update(); err != nil {
		return gerror.Wrap(err, "更新运维资产状态失败，请稍后重试！")
	}
	return nil
}
