package sys

import (
	"context"
	"hotgo/internal/dao"
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

type sSysOpsDevice struct{}

func NewSysOpsDevice() *sSysOpsDevice {
	return &sSysOpsDevice{}
}

func init() {
	service.RegisterSysOpsDevice(NewSysOpsDevice())
}

func (s *sSysOpsDevice) Model(ctx context.Context, option ...*handler.Option) *gdb.Model {
	return handler.Model(dao.OpsDevice.Ctx(ctx), option...)
}

func (s *sSysOpsDevice) List(ctx context.Context, in *sysin.OpsDeviceListInp) (list []*sysin.OpsDeviceListModel, totalCount int, err error) {
	mod := s.Model(ctx).Fields(sysin.OpsDeviceListModel{})

	if in.Id > 0 {
		mod = mod.Where(dao.OpsDevice.Columns().Id, in.Id)
	}
	if in.Name != "" {
		mod = mod.WhereLike(dao.OpsDevice.Columns().Name, "%"+in.Name+"%")
	}
	if in.Hostname != "" {
		mod = mod.WhereLike(dao.OpsDevice.Columns().Hostname, "%"+in.Hostname+"%")
	}
	if in.Ip != "" {
		mod = mod.WhereLike(dao.OpsDevice.Columns().Ip, "%"+in.Ip+"%")
	}
	if in.DeviceType != "" {
		mod = mod.Where(dao.OpsDevice.Columns().DeviceType, in.DeviceType)
	}
	if in.Status > 0 {
		mod = mod.Where(dao.OpsDevice.Columns().Status, in.Status)
	}
	if len(in.CreatedAt) == 2 {
		mod = mod.WhereBetween(dao.OpsDevice.Columns().CreatedAt, in.CreatedAt[0], in.CreatedAt[1])
	}

	mod = mod.Page(in.Page, in.PerPage)
	mod = mod.OrderAsc(dao.OpsDevice.Columns().Sort).OrderDesc(dao.OpsDevice.Columns().Id)

	if err = mod.ScanAndCount(&list, &totalCount, false); err != nil {
		return nil, 0, gerror.Wrap(err, "获取运维设备列表失败，请稍后重试！")
	}
	return
}

func (s *sSysOpsDevice) Edit(ctx context.Context, in *sysin.OpsDeviceEditInp) (err error) {
	data := do.OpsDevice{
		Name:       in.Name,
		Hostname:   in.Hostname,
		Ip:         in.Ip,
		DeviceType: in.DeviceType,
		OsName:     in.OsName,
		Location:   in.Location,
		Sort:       in.Sort,
		Remark:     in.Remark,
		Status:     in.Status,
	}

	return g.DB().Transaction(ctx, func(ctx context.Context, tx gdb.TX) (err error) {
		if in.Id > 0 {
			if _, err = s.Model(ctx).
				Fields(sysin.OpsDeviceUpdateFields{}).
				WherePri(in.Id).
				Data(data).
				Update(); err != nil {
				return gerror.Wrap(err, "修改运维设备失败，请稍后重试！")
			}
			return nil
		}

		if _, err = s.Model(ctx, &handler.Option{FilterAuth: false}).
			Fields(sysin.OpsDeviceInsertFields{}).
			Data(data).
			OmitEmptyData().
			Insert(); err != nil {
			return gerror.Wrap(err, "新增运维设备失败，请稍后重试！")
		}
		return nil
	})
}

func (s *sSysOpsDevice) Delete(ctx context.Context, in *sysin.OpsDeviceDeleteInp) (err error) {
	var ids = gconv.Uint64s(in.Id)
	if len(ids) > 0 {
		count, countErr := dao.OpsAsset.Ctx(ctx).WhereIn(dao.OpsAsset.Columns().DeviceId, ids).Count()
		if countErr != nil {
			return gerror.Wrap(countErr, "校验关联资产失败，请稍后重试！")
		}
		if count > 0 {
			return gerror.New("存在关联资产，请先删除相关资产后再删除设备")
		}
	}

	if _, err = s.Model(ctx).WherePri(in.Id).Unscoped().Delete(); err != nil {
		return gerror.Wrap(err, "删除运维设备失败，请稍后重试！")
	}
	return nil
}

func (s *sSysOpsDevice) MaxSort(ctx context.Context, in *sysin.OpsDeviceMaxSortInp) (res *sysin.OpsDeviceMaxSortModel, err error) {
	if err = dao.OpsDevice.Ctx(ctx).Fields(dao.OpsDevice.Columns().Sort).OrderDesc(dao.OpsDevice.Columns().Sort).Scan(&res); err != nil {
		return nil, gerror.Wrap(err, "获取运维设备最大排序失败，请稍后重试！")
	}
	if res == nil {
		res = new(sysin.OpsDeviceMaxSortModel)
	}
	res.Sort = form.DefaultMaxSort(res.Sort)
	return
}

func (s *sSysOpsDevice) View(ctx context.Context, in *sysin.OpsDeviceViewInp) (res *sysin.OpsDeviceViewModel, err error) {
	if err = s.Model(ctx).WherePri(in.Id).Scan(&res); err != nil {
		return nil, gerror.Wrap(err, "获取运维设备信息失败，请稍后重试！")
	}
	return
}

func (s *sSysOpsDevice) Status(ctx context.Context, in *sysin.OpsDeviceStatusInp) (err error) {
	if _, err = s.Model(ctx).
		WherePri(in.Id).
		Data(do.OpsDevice{Status: in.Status}).
		Update(); err != nil {
		return gerror.Wrap(err, "更新运维设备状态失败，请稍后重试！")
	}
	return nil
}

func (s *sSysOpsDevice) Option(ctx context.Context) (opts []*model.Option, err error) {
	var list []*entity.OpsDevice
	if err = s.Model(ctx, &handler.Option{FilterAuth: false}).
		Fields(dao.OpsDevice.Columns().Id, dao.OpsDevice.Columns().Name).
		Where(dao.OpsDevice.Columns().Status, 1).
		OrderAsc(dao.OpsDevice.Columns().Sort).
		OrderDesc(dao.OpsDevice.Columns().Id).
		Scan(&list); err != nil {
		return nil, gerror.Wrap(err, "获取运维设备选项失败，请稍后重试！")
	}

	opts = make([]*model.Option, 0, len(list))
	for _, item := range list {
		opts = append(opts, &model.Option{
			Key:   item.Id,
			Label: item.Name,
			Value: item.Id,
		})
	}
	return
}
