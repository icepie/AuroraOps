package sys

import (
	"context"
	"fmt"
	"sort"
	"strings"

	"auroraops/internal/consts"
	"auroraops/internal/dao"
	"auroraops/internal/model/entity"
	"auroraops/internal/model/input/sysin"
	"auroraops/internal/service"
	"auroraops/utility/convert"
	"auroraops/utility/excel"
	"github.com/gogf/gf/v2/os/gctx"
	"github.com/gogf/gf/v2/os/gtime"
	"github.com/gogf/gf/v2/util/gconv"
)

type sSysOpsHardware struct{}

func NewSysOpsHardware() *sSysOpsHardware {
	return &sSysOpsHardware{}
}

func init() {
	service.RegisterSysOpsHardware(NewSysOpsHardware())
}

func (s *sSysOpsHardware) Overview(ctx context.Context, in *sysin.OpsHardwareOverviewInp) (list []*sysin.OpsHardwareOverviewModel, totalCount int, err error) {
	var devices []*entity.OpsDevice
	deviceModel := dao.OpsDevice.Ctx(ctx)
	if in.GroupId > 0 {
		deviceModel = deviceModel.Where(dao.OpsDevice.Columns().GroupId, in.GroupId)
	}
	if in.Keyword != "" {
		deviceModel = deviceModel.WhereLike(dao.OpsDevice.Columns().Name, "%"+in.Keyword+"%")
	}
	if !in.IncludeDisabled {
		deviceModel = deviceModel.Where(dao.OpsDevice.Columns().Status, consts.StatusEnabled)
	}
	if err = deviceModel.
		OrderAsc(dao.OpsDevice.Columns().Sort).
		OrderDesc(dao.OpsDevice.Columns().Id).
		Scan(&devices); err != nil {
		return nil, 0, err
	}
	if len(devices) == 0 {
		return []*sysin.OpsHardwareOverviewModel{}, 0, nil
	}

	groupNameMap := map[uint64]string{}
	groupIds := make([]uint64, 0)
	for _, item := range devices {
		if item == nil || item.GroupId == 0 {
			continue
		}
		if _, ok := groupNameMap[item.GroupId]; ok {
			continue
		}
		groupIds = append(groupIds, item.GroupId)
		groupNameMap[item.GroupId] = ""
	}
	if len(groupIds) > 0 {
		var groups []*entity.OpsDeviceGroup
		if err = dao.OpsDeviceGroup.Ctx(ctx).WhereIn(dao.OpsDeviceGroup.Columns().Id, groupIds).Scan(&groups); err != nil {
			return nil, 0, err
		}
		for _, item := range groups {
			if item == nil {
				continue
			}
			groupNameMap[item.Id] = item.Name
		}
	}

	deviceIds := make([]uint64, 0, len(devices))
	for _, item := range devices {
		if item == nil {
			continue
		}
		deviceIds = append(deviceIds, item.Id)
	}

	var assets []*entity.OpsAsset
	assetModel := dao.OpsAsset.Ctx(ctx).WhereIn(dao.OpsAsset.Columns().DeviceId, deviceIds)
	if !in.IncludeDisabled {
		assetModel = assetModel.Where(dao.OpsAsset.Columns().Status, consts.StatusEnabled)
	}
	if err = assetModel.
		OrderAsc(dao.OpsAsset.Columns().Sort).
		OrderDesc(dao.OpsAsset.Columns().Id).
		Scan(&assets); err != nil {
		return nil, 0, err
	}

	assetMap := make(map[uint64][]*entity.OpsAsset, len(deviceIds))
	for _, item := range assets {
		if item == nil {
			continue
		}
		assetMap[item.DeviceId] = append(assetMap[item.DeviceId], item)
	}

	result := make([]*sysin.OpsHardwareOverviewModel, 0, len(devices))
	for _, device := range devices {
		if device == nil {
			continue
		}
		row := s.buildOverviewRow(device, groupNameMap[device.GroupId], assetMap[device.Id])
		if in.OnlyChanged && !row.HasChange {
			continue
		}
		result = append(result, row)
	}

	totalCount = len(result)
	paged, pageErr := s.pageOverviewRows(result, in.Page, in.PerPage)
	if pageErr != nil {
		return nil, 0, pageErr
	}
	return paged, totalCount, nil
}

func (s *sSysOpsHardware) Export(ctx context.Context, in *sysin.OpsHardwareOverviewInp) (err error) {
	exportInp := *in
	exportInp.Page = 1
	exportInp.PerPage = 100000

	list, totalCount, err := s.Overview(ctx, &exportInp)
	if err != nil {
		return err
	}

	tags, err := convert.GetEntityDescTags(sysin.OpsHardwareOverviewExportModel{})
	if err != nil {
		return err
	}

	var (
		fileName  = "导出硬件列表-" + gctx.CtxId(ctx)
		sheetName = fmt.Sprintf("筛选结果共%v行，本次导出%v行", totalCount, len(list))
		exports   []sysin.OpsHardwareOverviewExportModel
	)

	if err = gconv.Scan(list, &exports); err != nil {
		return err
	}

	return excel.ExportByStructs(ctx, tags, exports, fileName, sheetName)
}

func (s *sSysOpsHardware) buildOverviewRow(device *entity.OpsDevice, groupName string, assets []*entity.OpsAsset) *sysin.OpsHardwareOverviewModel {
	row := &sysin.OpsHardwareOverviewModel{
		DeviceId:   device.Id,
		DeviceName: device.Name,
		GroupId:    device.GroupId,
		GroupName:  groupName,
		Status:     device.Status,
	}
	if row.GroupName == "" {
		row.GroupName = "未分组"
	}

	memoryCapacity := make([]string, 0)
	cpuNames := make([]string, 0)
	nicNames := make([]string, 0)
	gpuNames := make([]string, 0)
	motherboardNames := make([]string, 0)
	biosHints := make([]string, 0)
	var changedAt *gtime.Time

	for _, asset := range assets {
		if asset == nil {
			continue
		}
		changedAt = laterTime(changedAt, asset.UpdatedAt)
		row.HasChange = true

		switch normalizeAssetType(asset.AssetType) {
		case "motherboard":
			motherboardNames = appendIfMeaningful(motherboardNames, composeAssetLabel(asset))
			biosHints = appendIfMeaningful(biosHints, extractBiosVersion(asset))
		case "bios":
			biosHints = appendIfMeaningful(biosHints, composeAssetLabel(asset))
		case "cpu":
			cpuNames = appendIfMeaningful(cpuNames, composeAssetLabel(asset))
		case "memory":
			memoryCapacity = appendIfMeaningful(memoryCapacity, composeMemoryLabel(asset))
		case "gpu":
			gpuNames = appendIfMeaningful(gpuNames, composeAssetLabel(asset))
		case "network":
			nicNames = appendIfMeaningful(nicNames, composeAssetLabel(asset))
		}
	}

	row.Motherboard = joinLabels(motherboardNames)
	row.BiosVersion = joinLabels(biosHints)
	row.Cpu = joinLabels(cpuNames)
	row.Memory = joinLabels(memoryCapacity)
	row.Gpu = joinLabels(gpuNames)
	row.Nic = joinLabels(nicNames)
	row.ChangedAt = changedAt
	return row
}

func (s *sSysOpsHardware) pageOverviewRows(list []*sysin.OpsHardwareOverviewModel, page, perPage int) ([]*sysin.OpsHardwareOverviewModel, error) {
	if perPage <= 0 {
		perPage = consts.DefaultPageSize
	}
	if page <= 0 {
		page = 1
	}
	start := (page - 1) * perPage
	if start >= len(list) {
		return []*sysin.OpsHardwareOverviewModel{}, nil
	}
	end := start + perPage
	if end > len(list) {
		end = len(list)
	}
	return list[start:end], nil
}

func normalizeAssetType(value string) string {
	switch strings.ToLower(strings.TrimSpace(value)) {
	case "nic", "network", "network_interface":
		return "network"
	case "video", "graphics", "graphics_card":
		return "gpu"
	default:
		return strings.ToLower(strings.TrimSpace(value))
	}
}

func composeAssetLabel(asset *entity.OpsAsset) string {
	if asset == nil {
		return ""
	}
	parts := make([]string, 0, 3)
	if text := strings.TrimSpace(asset.AssetName); text != "" {
		parts = append(parts, text)
	}
	if text := strings.TrimSpace(asset.Brand); text != "" && !containsText(parts, text) {
		parts = append(parts, text)
	}
	if text := strings.TrimSpace(asset.Model); text != "" && !containsText(parts, text) {
		parts = append(parts, text)
	}
	if len(parts) == 0 {
		return strings.TrimSpace(asset.Specification)
	}
	return strings.Join(parts, " / ")
}

func composeMemoryLabel(asset *entity.OpsAsset) string {
	label := composeAssetLabel(asset)
	spec := strings.TrimSpace(asset.Specification)
	if spec == "" {
		return label
	}
	if label == "" {
		return spec
	}
	if strings.Contains(label, spec) {
		return label
	}
	return label + " / " + spec
}

func extractBiosVersion(asset *entity.OpsAsset) string {
	if asset == nil {
		return ""
	}
	candidates := []string{
		strings.TrimSpace(asset.Model),
		strings.TrimSpace(asset.Specification),
		strings.TrimSpace(asset.AssetName),
	}
	for _, item := range candidates {
		if item == "" {
			continue
		}
		if strings.Contains(strings.ToLower(item), "bios") || looksLikeVersion(item) {
			return item
		}
	}
	return ""
}

func looksLikeVersion(value string) bool {
	value = strings.TrimSpace(value)
	if value == "" {
		return false
	}
	hasDigit := false
	hasDot := false
	for _, ch := range value {
		if ch >= '0' && ch <= '9' {
			hasDigit = true
		}
		if ch == '.' || ch == '-' || ch == '_' {
			hasDot = true
		}
	}
	return hasDigit && hasDot
}

func laterTime(a, b *gtime.Time) *gtime.Time {
	if a == nil {
		return b
	}
	if b == nil {
		return a
	}
	if b.TimestampMilli() > a.TimestampMilli() {
		return b
	}
	return a
}

func appendIfMeaningful(list []string, value string) []string {
	value = strings.TrimSpace(value)
	if value == "" || value == "-" {
		return list
	}
	for _, item := range list {
		if item == value {
			return list
		}
	}
	return append(list, value)
}

func containsText(list []string, value string) bool {
	for _, item := range list {
		if strings.EqualFold(strings.TrimSpace(item), strings.TrimSpace(value)) {
			return true
		}
	}
	return false
}

func joinLabels(values []string) string {
	if len(values) == 0 {
		return "-"
	}
	sort.Strings(values)
	return strings.Join(values, "；")
}
