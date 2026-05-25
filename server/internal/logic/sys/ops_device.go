package sys

import (
	"auroraops/internal/consts"
	"auroraops/internal/dao"
	"auroraops/internal/library/contexts"
	"auroraops/internal/library/hgorm/handler"
	"auroraops/internal/model"
	"auroraops/internal/model/do"
	"auroraops/internal/model/entity"
	"auroraops/internal/model/input/form"
	"auroraops/internal/model/input/sysin"
	"auroraops/internal/service"
	"auroraops/utility/encrypt"
	"context"
	"database/sql"
	"fmt"
	"net"
	"net/url"
	"strconv"
	"strings"

	"github.com/gogf/gf/v2/database/gdb"
	"github.com/gogf/gf/v2/errors/gerror"
	"github.com/gogf/gf/v2/frame/g"
	"github.com/gogf/gf/v2/net/ghttp"
	"github.com/gogf/gf/v2/os/gtime"
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
	mod := s.Model(ctx).As("d").
		LeftJoin(
			fmt.Sprintf("%s g", dao.OpsDeviceGroup.Table()),
			fmt.Sprintf("d.%s = g.%s", dao.OpsDevice.Columns().GroupId, dao.OpsDeviceGroup.Columns().Id),
		).
		Fields(
			"d.id",
			"d.group_id",
			"g.name as group_name",
			"d.name",
			"d.hostname",
			"d.ip",
			"d.device_type",
			"d.os_name",
			"d.location",
			"d.status",
			"d.created_at",
		)

	if in.Id > 0 {
		mod = mod.Where("d."+dao.OpsDevice.Columns().Id, in.Id)
	}
	if in.GroupScope == "ungrouped" {
		mod = mod.Where("d."+dao.OpsDevice.Columns().GroupId, 0)
	} else if in.GroupId > 0 {
		mod = mod.Where("d."+dao.OpsDevice.Columns().GroupId, in.GroupId)
	}
	if in.Name != "" {
		mod = mod.WhereLike("d."+dao.OpsDevice.Columns().Name, "%"+in.Name+"%")
	}
	if in.Hostname != "" {
		mod = mod.WhereLike("d."+dao.OpsDevice.Columns().Hostname, "%"+in.Hostname+"%")
	}
	if in.Ip != "" {
		mod = mod.WhereLike("d."+dao.OpsDevice.Columns().Ip, "%"+in.Ip+"%")
	}
	if in.DeviceType != "" {
		mod = mod.Where("d."+dao.OpsDevice.Columns().DeviceType, in.DeviceType)
	}
	if in.Status > 0 {
		mod = mod.Where("d."+dao.OpsDevice.Columns().Status, in.Status)
	}
	if len(in.CreatedAt) == 2 {
		mod = mod.WhereBetween("d."+dao.OpsDevice.Columns().CreatedAt, in.CreatedAt[0], in.CreatedAt[1])
	}

	mod = mod.Page(in.Page, in.PerPage)
	mod = mod.OrderAsc("d." + dao.OpsDevice.Columns().Sort).OrderDesc("d." + dao.OpsDevice.Columns().Id)

	if err = mod.ScanAndCount(&list, &totalCount, false); err != nil {
		return nil, 0, gerror.Wrap(err, "获取运维设备列表失败，请稍后重试！")
	}
	onlineSet := s.getOnlineDeviceIDs()
	for _, item := range list {
		if item == nil {
			continue
		}
		_, item.Online = onlineSet[item.Id]
	}
	return
}

func (s *sSysOpsDevice) getOnlineDeviceIDs() map[uint64]struct{} {
	onlineSet := make(map[uint64]struct{})
	clients := service.TCPServer().Instance().GetGroupClients("device")
	for _, client := range clients {
		if client == nil || client.Auth == nil {
			continue
		}
		if deviceID := parseOnlineDeviceID(client.Auth.AppId); deviceID > 0 {
			onlineSet[deviceID] = struct{}{}
			continue
		}
		if client.Auth.Extra == nil {
			continue
		}
		if deviceID := gconv.Uint64(client.Auth.Extra["deviceId"]); deviceID > 0 {
			onlineSet[deviceID] = struct{}{}
		}
	}
	return onlineSet
}

func parseOnlineDeviceID(appID string) uint64 {
	const prefix = "device:"
	if !strings.HasPrefix(appID, prefix) {
		return 0
	}
	deviceID, err := strconv.ParseUint(strings.TrimPrefix(appID, prefix), 10, 64)
	if err != nil {
		return 0
	}
	return deviceID
}

func (s *sSysOpsDevice) Edit(ctx context.Context, in *sysin.OpsDeviceEditInp) (err error) {
	if in.GroupId > 0 {
		exists, countErr := dao.OpsDeviceGroup.Ctx(ctx).WherePri(in.GroupId).Count()
		if countErr != nil {
			return gerror.Wrap(countErr, "校验设备分组失败，请稍后重试！")
		}
		if exists == 0 {
			return gerror.New("设备分组不存在，请重新选择")
		}
	}

	data := do.OpsDevice{
		GroupId:    in.GroupId,
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

func (s *sSysOpsDevice) CreateTerminalSession(ctx context.Context, in *sysin.OpsDeviceTerminalCreateInp) (res *sysin.OpsDeviceTerminalCreateModel, err error) {
	device := new(entity.OpsDevice)
	if err = dao.OpsDevice.Ctx(ctx).WherePri(in.DeviceId).Scan(device); err != nil {
		return nil, gerror.Wrap(err, "获取运维设备信息失败，请稍后重试！")
	}
	if device.Id == 0 {
		return nil, gerror.New("设备不存在")
	}
	if !s.isDeviceOnline(in.DeviceId) {
		return nil, gerror.New("设备离线，无法发起远程登录")
	}

	userID := contexts.GetUserId(ctx)
	sessionID, createErr := service.TCPServer().CreateTerminalSession(ctx, in.DeviceId, userID)
	if createErr != nil {
		return nil, gerror.New(createErr.Error())
	}

	res = &sysin.OpsDeviceTerminalCreateModel{
		SessionId: sessionID,
		WsPath:    fmt.Sprintf("/admin/opsDevice/terminal/ws?sessionId=%s", url.QueryEscape(sessionID)),
		PagePath:  fmt.Sprintf("/admin/#/ops/device/terminal?sessionId=%s&deviceId=%d&name=%s", url.QueryEscape(sessionID), in.DeviceId, url.QueryEscape(device.Name)),
	}
	return
}

func (s *sSysOpsDevice) CreateDesktopSession(ctx context.Context, in *sysin.OpsDeviceDesktopCreateInp) (res *sysin.OpsDeviceDesktopCreateModel, err error) {
	device := new(entity.OpsDevice)
	if err = dao.OpsDevice.Ctx(ctx).WherePri(in.DeviceId).Scan(device); err != nil {
		return nil, gerror.Wrap(err, "获取运维设备信息失败，请稍后重试！")
	}
	if device.Id == 0 {
		return nil, gerror.New("设备不存在")
	}
	if !s.isDeviceOnline(in.DeviceId) {
		return nil, gerror.New("设备离线，无法发起远程桌面")
	}

	userID := contexts.GetUserId(ctx)
	sessionID, createErr := service.TCPServer().CreateDesktopSession(ctx, in.DeviceId, userID)
	if createErr != nil {
		return nil, gerror.New(createErr.Error())
	}

	res = &sysin.OpsDeviceDesktopCreateModel{
		SessionId:  sessionID,
		WsPath:     fmt.Sprintf("/admin/opsDevice/desktop/ws?sessionId=%s", url.QueryEscape(sessionID)),
		PagePath:   fmt.Sprintf("/admin/#/ops/device/desktop?sessionId=%s&deviceId=%d&name=%s", url.QueryEscape(sessionID), in.DeviceId, url.QueryEscape(device.Name)),
		WeylusPath: fmt.Sprintf("/admin/opsDevice/weylus/?deviceId=%d&authorization=", in.DeviceId),
	}
	return
}

func (s *sSysOpsDevice) isDeviceOnline(deviceID uint64) bool {
	_, ok := s.getOnlineDeviceIDs()[deviceID]
	return ok
}

func (s *sSysOpsDevice) ClientRegister(ctx context.Context, in *sysin.OpsDeviceClientRegisterInp) (res *sysin.OpsDeviceClientRegisterModel, err error) {
	res = &sysin.OpsDeviceClientRegisterModel{
		Name:     in.Name,
		Hostname: in.Hostname,
		Ip:       in.Ip,
	}

	deviceType := in.DeviceType
	if deviceType == "" {
		deviceType = "physical"
	}

	location := in.Location

	nowText := gtime.Now().Format("Y-m-d H:i:s")

	err = g.DB().Transaction(ctx, func(ctx context.Context, _ gdb.TX) (err error) {
		var current entity.OpsDevice
		query := dao.OpsDevice.Ctx(ctx)

		if err = query.Where(dao.OpsDevice.Columns().Hostname, in.Hostname).Scan(&current); err != nil {
			if isNoRowsError(err) {
				err = nil
			} else {
				return gerror.Wrap(err, "查询设备信息失败，请稍后重试！")
			}
		}

		if current.Id > 0 {
			updateData := do.OpsDevice{
				Name:     in.Name,
				Hostname: in.Hostname,
				Ip:       in.Ip,
				OsName:   in.OsName,
			}

			if _, err = dao.OpsDevice.Ctx(ctx).
				WherePri(current.Id).
				Data(updateData).
				OmitEmptyData().
				Update(); err != nil {
				return gerror.Wrap(err, "更新客户端设备信息失败，请稍后重试！")
			}

			res.Id = current.Id
			res.Created = false
			res.CreatedAt = nowText
			res.Token, err = s.IssueClientToken(ctx, current.Id, in.Hostname)
			if err != nil {
				return err
			}
			res.TcpAddress = s.getTCPServerAddress(ctx)
			return nil
		}

		maxSort, err := s.MaxSort(ctx, &sysin.OpsDeviceMaxSortInp{})
		if err != nil {
			return err
		}

		insertData := do.OpsDevice{
			Name:       in.Name,
			Hostname:   in.Hostname,
			Ip:         in.Ip,
			DeviceType: deviceType,
			OsName:     in.OsName,
			Location:   location,
			Sort:       maxSort.Sort,
			Status:     consts.StatusEnabled,
			Remark:     "AuroraOps Client 自动注册",
		}

		result, err := dao.OpsDevice.Ctx(ctx).
			Fields(sysin.OpsDeviceInsertFields{}).
			Data(insertData).
			OmitEmptyData().
			InsertAndGetId()
		if err != nil {
			return gerror.Wrap(err, "注册客户端设备失败，请稍后重试！")
		}

		res.Id = gconv.Uint64(result)
		res.Created = true
		res.CreatedAt = nowText
		res.Token, err = s.IssueClientToken(ctx, res.Id, in.Hostname)
		if err != nil {
			return err
		}
		res.TcpAddress = s.getTCPServerAddress(ctx)
		return nil
	})
	return
}

func (s *sSysOpsDevice) ClientHeartbeat(ctx context.Context, in *sysin.OpsDeviceClientHeartbeatInp) (res *sysin.OpsDeviceClientHeartbeatModel, err error) {
	res = &sysin.OpsDeviceClientHeartbeatModel{
		Id:      in.Id,
		AliveAt: gtime.Now().Format("Y-m-d H:i:s"),
	}

	device := &entity.OpsDevice{}
	if err = dao.OpsDevice.Ctx(ctx).WherePri(in.Id).Scan(device); err != nil {
		if !isNoRowsError(err) {
			return nil, gerror.Wrap(err, "查询设备信息失败，请稍后重试！")
		}
	}
	if device.Id == 0 {
		return nil, gerror.New("设备不存在，请重新注册")
	}
	if device.Hostname != "" && device.Hostname != in.Hostname {
		return nil, gerror.New("设备主机名不匹配，请重新注册")
	}

	updateData := do.OpsDevice{
		Hostname: in.Hostname,
		Ip:       in.Ip,
		OsName:   in.OsName,
		Status:   consts.StatusEnabled,
	}
	if _, err = dao.OpsDevice.Ctx(ctx).
		WherePri(in.Id).
		Data(updateData).
		OmitEmptyData().
		Update(); err != nil {
		return nil, gerror.Wrap(err, "更新设备心跳失败，请稍后重试！")
	}

	return
}

func (s *sSysOpsDevice) IssueClientToken(ctx context.Context, deviceId uint64, hostname string) (token string, err error) {
	if deviceId == 0 {
		return "", gerror.New("设备ID不能为空")
	}
	hostname = strings.TrimSpace(hostname)
	if hostname == "" {
		return "", gerror.New("主机名不能为空")
	}

	secret := g.Cfg().MustGet(ctx, "token.secretKey").String()
	sign := encrypt.Md5ToString(fmt.Sprintf("%d:%s:%s", deviceId, hostname, secret))
	return fmt.Sprintf("%d:%s", deviceId, sign), nil
}

func (s *sSysOpsDevice) VerifyClientToken(ctx context.Context, deviceId uint64, hostname, token string) (err error) {
	expected, err := s.IssueClientToken(ctx, deviceId, hostname)
	if err != nil {
		return err
	}
	if token != expected {
		return gerror.New("设备令牌无效，请重新绑定")
	}
	return nil
}

func (s *sSysOpsDevice) getTCPServerAddress(ctx context.Context) string {
	address := g.Cfg().MustGet(ctx, "tcp.server.address").String()
	if strings.HasPrefix(address, ":") {
		request := ghttp.RequestFromCtx(ctx)
		if request != nil {
			host := request.GetHost()
			if parsedHost, _, err := net.SplitHostPort(host); err == nil && parsedHost != "" {
				return parsedHost + address
			}
			if host != "" {
				return host + address
			}
		}
		return "127.0.0.1" + address
	}
	return address
}

func isNoRowsError(err error) bool {
	if err == nil {
		return false
	}
	return gerror.Is(err, sql.ErrNoRows) || strings.Contains(err.Error(), "no rows in result set")
}
