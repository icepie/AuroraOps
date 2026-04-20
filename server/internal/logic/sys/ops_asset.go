package sys

import (
	"context"
	"crypto/sha1"
	"encoding/hex"
	"fmt"
	"hotgo/internal/consts"
	"hotgo/internal/dao"
	"hotgo/internal/library/hgorm/handler"
	"hotgo/internal/model/do"
	"hotgo/internal/model/entity"
	"hotgo/internal/model/input/form"
	"hotgo/internal/model/input/sysin"
	"hotgo/internal/service"
	"strings"

	"github.com/gogf/gf/v2/database/gdb"
	"github.com/gogf/gf/v2/errors/gerror"
	"github.com/gogf/gf/v2/frame/g"
	"github.com/gogf/gf/v2/os/gtime"
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
	fields, err := dao.OpsAsset.Ctx(ctx).TableFields(dao.OpsAsset.Table())
	if err != nil {
		return nil, 0, gerror.Wrap(err, "读取运维资产表结构失败，请稍后重试！")
	}

	selectFields := []any{
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
	}

	if _, ok := fields[dao.OpsAsset.Columns().Source]; ok {
		selectFields = append(selectFields, "a.source")
	} else {
		selectFields = append(selectFields, "'manual' as source")
	}

	if _, ok := fields[dao.OpsAsset.Columns().LastSeenAt]; ok {
		selectFields = append(selectFields, "a.last_seen_at")
	} else {
		selectFields = append(selectFields, "NULL as last_seen_at")
	}

	mod := s.Model(ctx).As("a").
		LeftJoin(
			fmt.Sprintf("%s d", dao.OpsDevice.Table()),
			fmt.Sprintf("a.%s = d.%s", dao.OpsAsset.Columns().DeviceId, dao.OpsDevice.Columns().Id),
		).
		Fields(selectFields...)

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
		Source:        in.Source,
		SyncHash:      in.SyncHash,
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

func (s *sSysOpsAsset) ClientSync(ctx context.Context, in *sysin.OpsAssetClientSyncInp) (res *sysin.OpsAssetClientSyncModel, err error) {
	res = &sysin.OpsAssetClientSyncModel{
		DeviceId: in.DeviceId,
		Total:    len(in.Assets),
	}

	device := &entity.OpsDevice{}
	if err = dao.OpsDevice.Ctx(ctx).WherePri(in.DeviceId).Scan(device); err != nil {
		return nil, gerror.Wrap(err, "查询设备失败，请稍后重试！")
	}
	if device.Id == 0 {
		return nil, gerror.New("所属设备不存在，请重新绑定")
	}

	err = g.DB().Transaction(ctx, func(ctx context.Context, tx gdb.TX) (err error) {
		var existing []*entity.OpsAsset
		if err = dao.OpsAsset.Ctx(ctx).
			Where(dao.OpsAsset.Columns().DeviceId, in.DeviceId).
			Scan(&existing); err != nil {
			return gerror.Wrap(err, "查询已有资产失败，请稍后重试！")
		}

		existingMap := make(map[string]*entity.OpsAsset, len(existing))
		duplicates := make([]*entity.OpsAsset, 0)
		for _, item := range existing {
			if item == nil {
				continue
			}
			key := s.assetSyncKey(item.AssetType, item.SerialNo)
			if _, ok := existingMap[key]; ok {
				duplicates = append(duplicates, item)
				continue
			}
			existingMap[key] = item
		}

		seen := make(map[string]struct{}, len(in.Assets))
		maxSort, sortErr := s.MaxSort(ctx, &sysin.OpsAssetMaxSortInp{})
		if sortErr != nil {
			return sortErr
		}
		nextSort := maxSort.Sort
		now := gtime.Now()

		for _, item := range in.Assets {
			key := s.assetSyncKey(item.AssetType, item.UniqueKey)
			seen[key] = struct{}{}
			syncHash := s.assetSyncHash(item)
			assetName := strings.TrimSpace(item.AssetName)
			if strings.TrimSpace(item.AssetType) == "network" && assetName == "Network Interface" {
				assetName = "网卡"
			}

			payload := do.OpsAsset{
				DeviceId:      in.DeviceId,
				AssetType:     item.AssetType,
				AssetName:     assetName,
				Brand:         item.Brand,
				Model:         item.Model,
				SerialNo:      item.UniqueKey,
				Specification: item.Specification,
				Source:        firstNonEmpty(strings.TrimSpace(item.Source), "agent"),
				SyncHash:      syncHash,
				LastSeenAt:    now,
				Remark:        item.Remark,
			}

			if current, ok := existingMap[key]; ok && current.Id > 0 {
				payload.Status = current.Status
				if strings.TrimSpace(current.Remark) != "" {
					payload.Remark = current.Remark
				}
				if _, err = dao.OpsAsset.Ctx(ctx).
					WherePri(current.Id).
					Data(payload).
					OmitEmptyData().
					Update(); err != nil {
					return gerror.Wrap(err, "更新资产失败，请稍后重试！")
				}
				res.Updated++
				continue
			}

			payload.Sort = nextSort
			payload.Status = consts.StatusEnabled
			nextSort++
			if _, err = dao.OpsAsset.Ctx(ctx).
				Fields(sysin.OpsAssetInsertFields{}).
				Data(payload).
				OmitEmptyData().
				Insert(); err != nil {
				return gerror.Wrap(err, "新增资产失败，请稍后重试！")
			}
			res.Created++
		}

		for _, item := range existing {
			if item == nil {
				continue
			}
			key := s.assetSyncKey(item.AssetType, item.SerialNo)
			if _, ok := seen[key]; ok {
				continue
			}
			if item.Status == consts.StatusDisable {
				continue
			}
			if _, err = dao.OpsAsset.Ctx(ctx).
				WherePri(item.Id).
				Data(do.OpsAsset{Status: consts.StatusDisable}).
				Update(); err != nil {
				return gerror.Wrap(err, "停用历史资产失败，请稍后重试！")
			}
			res.Disabled++
		}

		for _, item := range duplicates {
			if item == nil || item.Id == 0 {
				continue
			}
			if _, err = dao.OpsAsset.Ctx(ctx).WherePri(item.Id).Delete(); err != nil {
				return gerror.Wrap(err, "清理重复历史资产失败，请稍后重试！")
			}
		}
		return nil
	})
	return
}

func (s *sSysOpsAsset) ClientPull(ctx context.Context, in *sysin.OpsAssetClientPullInp) (res *sysin.OpsAssetClientPullModel, err error) {
	res = &sysin.OpsAssetClientPullModel{
		DeviceId: in.DeviceId,
		Assets:   make([]*sysin.OpsAssetClientPullItem, 0),
	}

	device := &entity.OpsDevice{}
	if err = dao.OpsDevice.Ctx(ctx).WherePri(in.DeviceId).Scan(device); err != nil {
		return nil, gerror.Wrap(err, "查询设备失败，请稍后重试！")
	}
	if device.Id == 0 {
		return nil, gerror.New("所属设备不存在，请重新绑定")
	}

	var list []*entity.OpsAsset
	if err = dao.OpsAsset.Ctx(ctx).
		Where(dao.OpsAsset.Columns().DeviceId, in.DeviceId).
		OrderAsc(dao.OpsAsset.Columns().Sort).
		OrderDesc(dao.OpsAsset.Columns().Id).
		Scan(&list); err != nil {
		return nil, gerror.Wrap(err, "查询服务端资产失败，请稍后重试！")
	}

	for _, item := range list {
		if item == nil {
			continue
		}
		res.Assets = append(res.Assets, &sysin.OpsAssetClientPullItem{
			Id:            item.Id,
			AssetType:     item.AssetType,
			UniqueKey:     item.SerialNo,
			AssetName:     item.AssetName,
			Brand:         item.Brand,
			Model:         item.Model,
			SerialNo:      item.SerialNo,
			Specification: item.Specification,
			Source:        item.Source,
			SyncHash:      item.SyncHash,
			Remark:        item.Remark,
			Status:        item.Status,
			LastSeenAt:    item.LastSeenAt,
		})
	}
	return
}

func (s *sSysOpsAsset) assetSyncKey(assetType, uniqueKey string) string {
	return strings.TrimSpace(assetType) + "::" + strings.TrimSpace(uniqueKey)
}

func (s *sSysOpsAsset) assetSyncHash(item *sysin.OpsAssetSyncItem) string {
	if item == nil {
		return ""
	}
	raw := strings.Join([]string{
		strings.TrimSpace(item.AssetType),
		strings.TrimSpace(item.UniqueKey),
		strings.TrimSpace(item.AssetName),
		strings.TrimSpace(item.Brand),
		strings.TrimSpace(item.Model),
		strings.TrimSpace(item.Specification),
	}, "|")
	sum := sha1.Sum([]byte(raw))
	return hex.EncodeToString(sum[:])
}

func firstNonEmpty(values ...string) string {
	for _, value := range values {
		value = strings.TrimSpace(value)
		if value != "" {
			return value
		}
	}
	return ""
}
