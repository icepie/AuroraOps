package sys

import (
	"context"
	"fmt"
	"hotgo/internal/dao"
	"hotgo/internal/library/dict"
	"hotgo/internal/library/hgorm/handler"
	"hotgo/internal/model"
	"hotgo/internal/model/do"
	"hotgo/internal/model/entity"
	"hotgo/internal/model/input/form"
	"hotgo/internal/model/input/sysin"
	"hotgo/internal/service"

	"github.com/gogf/gf/v2/database/gdb"
	"github.com/gogf/gf/v2/errors/gerror"
	"github.com/gogf/gf/v2/frame/g"
	"github.com/gogf/gf/v2/util/gconv"
)

type sSysOpsDeviceGroup struct{}

func NewSysOpsDeviceGroup() *sSysOpsDeviceGroup {
	return &sSysOpsDeviceGroup{}
}

func init() {
	service.RegisterSysOpsDeviceGroup(NewSysOpsDeviceGroup())
}

func (s *sSysOpsDeviceGroup) Model(ctx context.Context, option ...*handler.Option) *gdb.Model {
	return handler.Model(dao.OpsDeviceGroup.Ctx(ctx), option...)
}

func (s *sSysOpsDeviceGroup) List(ctx context.Context) (list []*sysin.OpsDeviceGroupListModel, err error) {
	mod := s.Model(ctx).As("g").
		LeftJoin(
			fmt.Sprintf("%s d", dao.OpsDevice.Table()),
			fmt.Sprintf(
				"g.%s = d.%s AND d.%s IS NULL",
				dao.OpsDeviceGroup.Columns().Id,
				dao.OpsDevice.Columns().GroupId,
				dao.OpsDevice.Columns().DeletedAt,
			),
		).
		Fields(
			"g.id",
			"g.name",
			"g.sort",
			"g.remark",
			"g.status",
			"g.created_at",
			"COUNT(d.id) AS device_count",
		).
		Group("g.id").
		OrderAsc("g." + dao.OpsDeviceGroup.Columns().Sort).
		OrderDesc("g." + dao.OpsDeviceGroup.Columns().Id)

	if err = mod.Scan(&list); err != nil {
		return nil, gerror.Wrap(err, "获取设备分组列表失败，请稍后重试！")
	}
	return
}

func (s *sSysOpsDeviceGroup) Edit(ctx context.Context, in *sysin.OpsDeviceGroupEditInp) (err error) {
	data := do.OpsDeviceGroup{
		Name:   in.Name,
		Sort:   in.Sort,
		Remark: in.Remark,
		Status: in.Status,
	}

	return g.DB().Transaction(ctx, func(ctx context.Context, tx gdb.TX) (err error) {
		if in.Id > 0 {
			if _, err = s.Model(ctx).
				Fields(sysin.OpsDeviceGroupUpdateFields{}).
				WherePri(in.Id).
				Data(data).
				Update(); err != nil {
				return gerror.Wrap(err, "修改设备分组失败，请稍后重试！")
			}
			return nil
		}

		if _, err = s.Model(ctx, &handler.Option{FilterAuth: false}).
			Fields(sysin.OpsDeviceGroupInsertFields{}).
			Data(data).
			OmitEmptyData().
			Insert(); err != nil {
			return gerror.Wrap(err, "新增设备分组失败，请稍后重试！")
		}
		return nil
	})
}

func (s *sSysOpsDeviceGroup) Delete(ctx context.Context, in *sysin.OpsDeviceGroupDeleteInp) (err error) {
	ids := gconv.Uint64s(in.Id)
	if len(ids) > 0 {
		count, countErr := dao.OpsDevice.Ctx(ctx).WhereIn(dao.OpsDevice.Columns().GroupId, ids).Count()
		if countErr != nil {
			return gerror.Wrap(countErr, "校验分组下设备失败，请稍后重试！")
		}
		if count > 0 {
			return gerror.New("分组下仍有关联设备，请先调整设备分组后再删除")
		}
	}

	if _, err = s.Model(ctx).WherePri(in.Id).Unscoped().Delete(); err != nil {
		return gerror.Wrap(err, "删除设备分组失败，请稍后重试！")
	}
	return nil
}

func (s *sSysOpsDeviceGroup) MaxSort(ctx context.Context, in *sysin.OpsDeviceGroupMaxSortInp) (res *sysin.OpsDeviceGroupMaxSortModel, err error) {
	if err = dao.OpsDeviceGroup.Ctx(ctx).
		Fields(dao.OpsDeviceGroup.Columns().Sort).
		OrderDesc(dao.OpsDeviceGroup.Columns().Sort).
		Scan(&res); err != nil {
		return nil, gerror.Wrap(err, "获取设备分组最大排序失败，请稍后重试！")
	}
	if res == nil {
		res = new(sysin.OpsDeviceGroupMaxSortModel)
	}
	res.Sort = form.DefaultMaxSort(res.Sort)
	return
}

func (s *sSysOpsDeviceGroup) View(ctx context.Context, in *sysin.OpsDeviceGroupViewInp) (res *sysin.OpsDeviceGroupViewModel, err error) {
	if err = s.Model(ctx).WherePri(in.Id).Scan(&res); err != nil {
		return nil, gerror.Wrap(err, "获取设备分组信息失败，请稍后重试！")
	}
	return
}

func (s *sSysOpsDeviceGroup) Status(ctx context.Context, in *sysin.OpsDeviceGroupStatusInp) (err error) {
	if _, err = s.Model(ctx).
		WherePri(in.Id).
		Data(do.OpsDeviceGroup{Status: in.Status}).
		Update(); err != nil {
		return gerror.Wrap(err, "更新设备分组状态失败，请稍后重试！")
	}
	return nil
}

func (s *sSysOpsDeviceGroup) Option(ctx context.Context) (opts []*model.Option, err error) {
	var list []*entity.OpsDeviceGroup
	if err = s.Model(ctx, &handler.Option{FilterAuth: false}).
		Fields(dao.OpsDeviceGroup.Columns().Id, dao.OpsDeviceGroup.Columns().Name).
		Where(dao.OpsDeviceGroup.Columns().Status, 1).
		OrderAsc(dao.OpsDeviceGroup.Columns().Sort).
		OrderDesc(dao.OpsDeviceGroup.Columns().Id).
		Scan(&list); err != nil {
		return nil, gerror.Wrap(err, "获取设备分组选项失败，请稍后重试！")
	}

	opts = make([]*model.Option, 0, len(list))
	for _, item := range list {
		opts = append(opts, dict.GenHashOption(item.Id, item.Name))
	}
	return
}
