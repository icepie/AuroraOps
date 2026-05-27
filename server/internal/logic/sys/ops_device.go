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
	"encoding/json"
	"fmt"
	"net"
	"net/url"
	"regexp"
	"strings"

	"github.com/gogf/gf/v2/database/gdb"
	"github.com/gogf/gf/v2/errors/gerror"
	"github.com/gogf/gf/v2/frame/g"
	"github.com/gogf/gf/v2/net/ghttp"
	"github.com/gogf/gf/v2/os/gtime"
	"github.com/gogf/gf/v2/util/gconv"
)

type sSysOpsDevice struct{}

type opsDeviceSchema struct {
	HasMacAddress    bool
	HasKernelVersion bool
}

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
	schema, err := s.deviceSchema(ctx)
	if err != nil {
		return nil, 0, err
	}
	fields := []any{
		"d.id",
		"d.group_id",
		"g.name as group_name",
		"d.name",
		"d.hostname",
		"d.ip",
		"d.device_type",
		"d.os_name",
		"COALESCE(NULLIF(d.architecture, ''), host_asset.model) as architecture",
		"d.location",
		"d.monitor_snapshot",
		"d.monitor_reported_at",
		"d.status",
		"d.created_at",
	}
	if schema.HasMacAddress {
		fields = append(fields, "d.mac_address")
	}
	if schema.HasKernelVersion {
		fields = append(fields, "d.kernel_version")
	}
	mod := s.Model(ctx).As("d").
		LeftJoin(
			fmt.Sprintf("%s g", dao.OpsDeviceGroup.Table()),
			fmt.Sprintf("d.%s = g.%s", dao.OpsDevice.Columns().GroupId, dao.OpsDeviceGroup.Columns().Id),
		).
		LeftJoin(
			fmt.Sprintf("%s host_asset", dao.OpsAsset.Table()),
			"host_asset.device_id = d.id AND host_asset.asset_type = 'host' AND host_asset.deleted_at IS NULL AND host_asset.status = 1",
		).
		Fields(fields...)

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
	if schema.HasMacAddress && in.MacAddress != "" {
		mod = mod.WhereLike("d."+dao.OpsDevice.Columns().MacAddress, "%"+in.MacAddress+"%")
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
	onlineSet := service.TCPServer().OnlineDeviceIDs()
	for _, item := range list {
		if item == nil {
			continue
		}
		_, item.Online = onlineSet[item.Id]
		s.normalizeDeviceListFields(item)
		if item.MacAddress == "" {
			item.MacAddress = s.findDeviceNetworkMac(ctx, item.Id)
		}
	}
	return
}

func (s *sSysOpsDevice) normalizeDeviceListFields(item *sysin.OpsDeviceListModel) {
	item.Architecture = normalizeDeviceArchitecture(item.Architecture, item.Location)
	item.Location = normalizeDeviceLocation(item.Location)
	item.Monitor = parseDeviceMonitorSnapshot(item.MonitorSnapshot)
	item.MonitorSnapshot = ""
}

func parseDeviceMonitorSnapshot(value string) *sysin.OpsDeviceMonitorView {
	value = strings.TrimSpace(value)
	if value == "" {
		return nil
	}
	var monitor sysin.OpsDeviceMonitorView
	if err := json.Unmarshal([]byte(value), &monitor); err != nil {
		return nil
	}
	return &monitor
}

func normalizeDeviceArchitecture(values ...string) string {
	for _, value := range values {
		value = strings.TrimSpace(value)
		if isArchitectureValue(value) {
			return value
		}
	}
	return ""
}

func normalizeDeviceLocation(value string) string {
	value = strings.TrimSpace(value)
	if isArchitectureValue(value) {
		return ""
	}
	return value
}

func normalizeDeviceType(value string) string {
	switch strings.ToLower(strings.TrimSpace(value)) {
	case "virtual", "vm", "virtual_machine", "virtual-machine", "hypervisor", "cloud":
		return "virtual"
	default:
		return "physical"
	}
}

func isArchitectureValue(value string) bool {
	switch strings.ToLower(strings.TrimSpace(value)) {
	case "aarch64", "arm64", "amd64", "x86_64", "i386", "i686", "loongarch64", "mips64", "mips64el", "sw_64", "riscv64":
		return true
	default:
		return false
	}
}

func (s *sSysOpsDevice) Edit(ctx context.Context, in *sysin.OpsDeviceEditInp) (err error) {
	schema, err := s.deviceSchema(ctx)
	if err != nil {
		return err
	}
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
		GroupId:       in.GroupId,
		Name:          in.Name,
		Hostname:      in.Hostname,
		Ip:            in.Ip,
		MacAddress:    normalizeMacAddress(in.MacAddress),
		DeviceType:    normalizeDeviceType(in.DeviceType),
		OsName:        in.OsName,
		Architecture:  normalizeDeviceArchitecture(in.Architecture),
		KernelVersion: strings.TrimSpace(in.KernelVersion),
		Location:      in.Location,
		Sort:          in.Sort,
		Remark:        in.Remark,
		Status:        in.Status,
	}

	return g.DB().Transaction(ctx, func(ctx context.Context, tx gdb.TX) (err error) {
		if in.Id > 0 {
			if _, err = s.Model(ctx).
				Fields(s.deviceUpdateFields(schema)...).
				WherePri(in.Id).
				Data(data).
				Update(); err != nil {
				return gerror.Wrap(err, "修改运维设备失败，请稍后重试！")
			}
			return nil
		}

		if _, err = s.Model(ctx, &handler.Option{FilterAuth: false}).
			Fields(s.deviceInsertFields(schema)...).
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
	schema, err := s.deviceSchema(ctx)
	if err != nil {
		return nil, err
	}
	mod := s.Model(ctx).WherePri(in.Id)
	if !schema.HasMacAddress {
		mod = mod.FieldsEx(dao.OpsDevice.Columns().MacAddress)
	}
	if err = mod.Scan(&res); err != nil {
		return nil, gerror.Wrap(err, "获取运维设备信息失败，请稍后重试！")
	}
	if res != nil && strings.TrimSpace(res.MacAddress) == "" {
		res.MacAddress = s.findDeviceNetworkMac(ctx, res.Id)
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

func (s *sSysOpsDevice) deviceSchema(ctx context.Context) (opsDeviceSchema, error) {
	fields, err := dao.OpsDevice.Ctx(ctx).TableFields(dao.OpsDevice.Table())
	if err != nil {
		return opsDeviceSchema{}, gerror.Wrap(err, "读取设备表结构失败，请稍后重试！")
	}
	if !hasTableField(fields, dao.OpsDevice.Columns().KernelVersion) {
		if err = s.ensureKernelVersionColumn(ctx); err == nil {
			fields, err = dao.OpsDevice.Ctx(ctx).TableFields(dao.OpsDevice.Table())
			if err != nil {
				return opsDeviceSchema{}, gerror.Wrap(err, "读取设备表结构失败，请稍后重试！")
			}
		} else {
			g.Log().Warningf(ctx, "ensure ops device kernel_version column failed: %v", err)
		}
	}
	return opsDeviceSchema{
		HasMacAddress:    hasTableField(fields, dao.OpsDevice.Columns().MacAddress),
		HasKernelVersion: hasTableField(fields, dao.OpsDevice.Columns().KernelVersion),
	}, nil
}

func (s *sSysOpsDevice) ensureKernelVersionColumn(ctx context.Context) error {
	db := g.DB()
	table := dao.OpsDevice.Table()
	column := dao.OpsDevice.Columns().KernelVersion
	var sql string
	switch strings.ToLower(db.GetConfig().Type) {
	case consts.DBPgsql:
		sql = fmt.Sprintf(
			`ALTER TABLE "%s" ADD COLUMN "%s" varchar(128) NOT NULL DEFAULT ''`,
			table,
			column,
		)
	case "sqlite":
		sql = fmt.Sprintf(
			`ALTER TABLE "%s" ADD COLUMN "%s" varchar(128) NOT NULL DEFAULT ''`,
			table,
			column,
		)
	default:
		sql = fmt.Sprintf(
			"ALTER TABLE `%s` ADD COLUMN `%s` varchar(128) NOT NULL DEFAULT '' COMMENT '内核版本' AFTER `%s`",
			table,
			column,
			dao.OpsDevice.Columns().Architecture,
		)
	}
	_, err := db.Exec(ctx, sql)
	return err
}

func (s *sSysOpsDevice) deviceBaseWriteFields(schema opsDeviceSchema) []any {
	fields := []any{
		dao.OpsDevice.Columns().GroupId,
		dao.OpsDevice.Columns().Name,
		dao.OpsDevice.Columns().Hostname,
		dao.OpsDevice.Columns().Ip,
		dao.OpsDevice.Columns().DeviceType,
		dao.OpsDevice.Columns().OsName,
		dao.OpsDevice.Columns().Architecture,
		dao.OpsDevice.Columns().Location,
		dao.OpsDevice.Columns().Sort,
		dao.OpsDevice.Columns().Remark,
		dao.OpsDevice.Columns().Status,
	}
	if schema.HasMacAddress {
		fields = append(fields, dao.OpsDevice.Columns().MacAddress)
	}
	if schema.HasKernelVersion {
		fields = append(fields, dao.OpsDevice.Columns().KernelVersion)
	}
	return fields
}

func (s *sSysOpsDevice) deviceUpdateFields(schema opsDeviceSchema) []any {
	return s.deviceBaseWriteFields(schema)
}

func (s *sSysOpsDevice) deviceInsertFields(schema opsDeviceSchema) []any {
	return s.deviceBaseWriteFields(schema)
}

func (s *sSysOpsDevice) deviceRegisterFields(schema opsDeviceSchema) []any {
	fields := []any{
		dao.OpsDevice.Columns().Name,
		dao.OpsDevice.Columns().Hostname,
		dao.OpsDevice.Columns().Ip,
		dao.OpsDevice.Columns().DeviceType,
		dao.OpsDevice.Columns().OsName,
		dao.OpsDevice.Columns().Architecture,
	}
	if schema.HasMacAddress {
		fields = append(fields, dao.OpsDevice.Columns().MacAddress)
	}
	if schema.HasKernelVersion {
		fields = append(fields, dao.OpsDevice.Columns().KernelVersion)
	}
	return fields
}

func (s *sSysOpsDevice) deviceHeartbeatFields(schema opsDeviceSchema) []any {
	fields := []any{
		dao.OpsDevice.Columns().Hostname,
		dao.OpsDevice.Columns().Ip,
		dao.OpsDevice.Columns().OsName,
		dao.OpsDevice.Columns().Architecture,
		dao.OpsDevice.Columns().Status,
	}
	if schema.HasMacAddress {
		fields = append(fields, dao.OpsDevice.Columns().MacAddress)
	}
	if schema.HasKernelVersion {
		fields = append(fields, dao.OpsDevice.Columns().KernelVersion)
	}
	return fields
}

func (s *sSysOpsDevice) findDeviceNetworkMac(ctx context.Context, deviceID uint64) string {
	if deviceID == 0 {
		return ""
	}
	var assets []*entity.OpsAsset
	if err := dao.OpsAsset.Ctx(ctx).
		Where(dao.OpsAsset.Columns().DeviceId, deviceID).
		Where(dao.OpsAsset.Columns().AssetType, "network").
		Where(dao.OpsAsset.Columns().Status, consts.StatusEnabled).
		OrderDesc(dao.OpsAsset.Columns().LastSeenAt).
		OrderDesc(dao.OpsAsset.Columns().Id).
		Limit(16).
		Scan(&assets); err != nil {
		return ""
	}
	for _, asset := range assets {
		if asset == nil {
			continue
		}
		if mac := normalizeMacAddress(firstMacCandidate(
			asset.SerialNo,
			asset.Model,
			asset.Specification,
			asset.UniqueKey,
			asset.Remark,
		)); mac != "" {
			return mac
		}
	}
	return ""
}

func firstMacCandidate(values ...string) string {
	for _, value := range values {
		if mac := extractFirstMac(value); mac != "" {
			return mac
		}
	}
	return ""
}

func extractFirstMac(value string) string {
	value = strings.TrimSpace(value)
	if value == "" {
		return ""
	}
	patterns := []*regexp.Regexp{
		regexp.MustCompile(`(?i)(?:[0-9a-f]{2}[:-]){5}[0-9a-f]{2}`),
		regexp.MustCompile(`(?i)[0-9a-f]{4}\.[0-9a-f]{4}\.[0-9a-f]{4}`),
		regexp.MustCompile(`(?i)\b[0-9a-f]{12}\b`),
	}
	for _, pattern := range patterns {
		if match := pattern.FindString(value); match != "" {
			return match
		}
	}
	return ""
}

func normalizeMacAddress(value string) string {
	mac, err := parseWakeMac(value)
	if err != nil {
		return strings.TrimSpace(value)
	}
	return formatWakeMac(mac)
}

func parseWakeMac(value string) ([6]byte, error) {
	var out [6]byte
	value = strings.TrimSpace(value)
	if value == "" {
		return out, gerror.New("MAC地址不能为空，无法发送WOL魔术包")
	}
	parsed, err := net.ParseMAC(value)
	if err != nil || len(parsed) != 6 {
		hexOnly := regexp.MustCompile(`(?i)[^0-9a-f]`).ReplaceAllString(value, "")
		if len(hexOnly) != 12 {
			return out, gerror.New("MAC地址格式不正确")
		}
		parsed = make([]byte, 6)
		for i := 0; i < 6; i++ {
			if _, err = fmt.Sscanf(hexOnly[i*2:i*2+2], "%02x", &parsed[i]); err != nil {
				return out, gerror.New("MAC地址格式不正确")
			}
		}
	}
	copy(out[:], parsed[:6])
	return out, nil
}

func formatWakeMac(mac [6]byte) string {
	return fmt.Sprintf("%02X:%02X:%02X:%02X:%02X:%02X", mac[0], mac[1], mac[2], mac[3], mac[4], mac[5])
}

func buildMagicPacket(mac [6]byte) []byte {
	packet := make([]byte, 6+16*6)
	for i := 0; i < 6; i++ {
		packet[i] = 0xFF
	}
	for i := 0; i < 16; i++ {
		copy(packet[6+i*6:6+(i+1)*6], mac[:])
	}
	return packet
}

func buildWakeTargets(broadcast, deviceIP string, port int) []string {
	seen := make(map[string]struct{})
	targets := make([]string, 0, 3)
	add := func(host string) {
		host = strings.TrimSpace(host)
		if host == "" {
			return
		}
		if _, _, err := net.SplitHostPort(host); err != nil {
			host = net.JoinHostPort(host, fmt.Sprintf("%d", port))
		}
		if _, ok := seen[host]; ok {
			return
		}
		seen[host] = struct{}{}
		targets = append(targets, host)
	}
	add(broadcast)
	if ip := net.ParseIP(deviceIP).To4(); ip != nil {
		add(fmt.Sprintf("%d.%d.%d.255", ip[0], ip[1], ip[2]))
	}
	add("255.255.255.255")
	return targets
}

func sendWakePacket(target string, packet []byte) error {
	addr, err := net.ResolveUDPAddr("udp4", target)
	if err != nil {
		return err
	}
	conn, err := net.ListenUDP("udp4", nil)
	if err != nil {
		return err
	}
	defer conn.Close()
	_, err = conn.WriteToUDP(packet, addr)
	return err
}

func (s *sSysOpsDevice) Wake(ctx context.Context, in *sysin.OpsDeviceWakeInp) (res *sysin.OpsDeviceWakeModel, err error) {
	device := new(entity.OpsDevice)
	schema, err := s.deviceSchema(ctx)
	if err != nil {
		return nil, err
	}
	mod := dao.OpsDevice.Ctx(ctx).WherePri(in.Id)
	if !schema.HasMacAddress {
		mod = mod.FieldsEx(dao.OpsDevice.Columns().MacAddress)
	}
	if err = mod.Scan(device); err != nil {
		return nil, gerror.Wrap(err, "获取运维设备信息失败，请稍后重试！")
	}
	if device.Id == 0 {
		return nil, gerror.New("设备不存在")
	}

	macText := firstNonEmpty(strings.TrimSpace(in.MacAddress), strings.TrimSpace(device.MacAddress))
	if macText == "" {
		macText = s.findDeviceNetworkMac(ctx, device.Id)
	}
	mac, err := parseWakeMac(macText)
	if err != nil {
		return nil, err
	}

	port := in.Port
	if port == 0 {
		port = 9
	}
	repeat := in.Repeat
	if repeat == 0 {
		repeat = 3
	}
	targets := buildWakeTargets(strings.TrimSpace(in.Broadcast), strings.TrimSpace(device.Ip), port)
	packet := buildMagicPacket(mac)
	sentTargets := make([]string, 0, len(targets))
	packets := 0
	var sendErrs []string

	for _, target := range targets {
		if target == "" {
			continue
		}
		ok := false
		var lastErr error
		for i := 0; i < repeat; i++ {
			if err = sendWakePacket(target, packet); err != nil {
				lastErr = err
				continue
			}
			ok = true
			packets++
		}
		if ok {
			sentTargets = append(sentTargets, target)
		} else if lastErr != nil {
			sendErrs = append(sendErrs, fmt.Sprintf("%s: %v", target, lastErr))
		}
	}
	if packets == 0 {
		return nil, gerror.New("WOL魔术包发送失败：" + strings.Join(sendErrs, "; "))
	}

	return &sysin.OpsDeviceWakeModel{
		Id:         device.Id,
		MacAddress: formatWakeMac(mac),
		Targets:    sentTargets,
		Packets:    packets,
	}, nil
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
	return service.TCPServer().IsDeviceOnline(deviceID)
}

func (s *sSysOpsDevice) ClientRegister(ctx context.Context, in *sysin.OpsDeviceClientRegisterInp) (res *sysin.OpsDeviceClientRegisterModel, err error) {
	schema, err := s.deviceSchema(ctx)
	if err != nil {
		return nil, err
	}
	res = &sysin.OpsDeviceClientRegisterModel{
		Name:       in.Name,
		Hostname:   in.Hostname,
		Ip:         in.Ip,
		MacAddress: normalizeMacAddress(in.MacAddress),
	}

	deviceType := normalizeDeviceType(in.DeviceType)

	architecture := normalizeDeviceArchitecture(in.Architecture, in.Location)
	location := normalizeDeviceLocation(in.Location)

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
				Name:          in.Name,
				Hostname:      in.Hostname,
				Ip:            in.Ip,
				MacAddress:    normalizeMacAddress(in.MacAddress),
				DeviceType:    deviceType,
				OsName:        in.OsName,
				Architecture:  architecture,
				KernelVersion: strings.TrimSpace(in.KernelVersion),
			}

			if _, err = dao.OpsDevice.Ctx(ctx).
				WherePri(current.Id).
				Fields(s.deviceRegisterFields(schema)...).
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
			Name:          in.Name,
			Hostname:      in.Hostname,
			Ip:            in.Ip,
			MacAddress:    normalizeMacAddress(in.MacAddress),
			DeviceType:    deviceType,
			OsName:        in.OsName,
			Architecture:  architecture,
			KernelVersion: strings.TrimSpace(in.KernelVersion),
			Location:      location,
			Sort:          maxSort.Sort,
			Status:        consts.StatusEnabled,
			Remark:        "AuroraOps Client 自动注册",
		}

		result, err := dao.OpsDevice.Ctx(ctx).
			Fields(s.deviceInsertFields(schema)...).
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
	schema, err := s.deviceSchema(ctx)
	if err != nil {
		return nil, err
	}
	mod := dao.OpsDevice.Ctx(ctx).WherePri(in.Id)
	if !schema.HasMacAddress {
		mod = mod.FieldsEx(dao.OpsDevice.Columns().MacAddress)
	}
	if err = mod.Scan(device); err != nil {
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
		Hostname:      in.Hostname,
		Ip:            in.Ip,
		MacAddress:    normalizeMacAddress(in.MacAddress),
		OsName:        in.OsName,
		Architecture:  normalizeDeviceArchitecture(in.Architecture, device.Architecture, device.Location),
		KernelVersion: strings.TrimSpace(in.KernelVersion),
		Status:        consts.StatusEnabled,
	}
	if _, err = dao.OpsDevice.Ctx(ctx).
		WherePri(in.Id).
		Fields(s.deviceHeartbeatFields(schema)...).
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
