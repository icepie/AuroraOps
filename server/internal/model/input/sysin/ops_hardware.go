package sysin

import (
	"context"
	"strings"

	"auroraops/internal/model/input/form"
	"github.com/gogf/gf/v2/os/gtime"
)

type OpsHardwareOverviewInp struct {
	form.PageReq
	GroupId         uint64 `json:"groupId"         dc:"分组ID"`
	Keyword         string `json:"keyword"         dc:"机器名关键词"`
	OnlyChanged     bool   `json:"onlyChanged"     dc:"仅显示变动项"`
	IncludeDisabled bool   `json:"includeDisabled" dc:"包含禁用设备"`
}

func (in *OpsHardwareOverviewInp) Filter(ctx context.Context) (err error) {
	in.Keyword = strings.TrimSpace(in.Keyword)
	return nil
}

type OpsHardwareOverviewModel struct {
	DeviceId    uint64      `json:"deviceId"    dc:"设备ID"`
	DeviceName  string      `json:"deviceName"  dc:"机器名"`
	GroupId     uint64      `json:"groupId"     dc:"分组ID"`
	GroupName   string      `json:"groupName"   dc:"分组"`
	Motherboard string      `json:"motherboard" dc:"主板"`
	BiosVersion string      `json:"biosVersion" dc:"BIOS版本"`
	Cpu         string      `json:"cpu"         dc:"CPU"`
	Memory      string      `json:"memory"      dc:"内存"`
	Gpu         string      `json:"gpu"         dc:"显卡"`
	Nic         string      `json:"nic"         dc:"网卡"`
	ChangedAt   *gtime.Time `json:"changedAt"   dc:"变更时间"`
	HasChange   bool        `json:"hasChange"   dc:"是否有变动"`
	Status      int         `json:"status"      dc:"状态"`
}

type OpsHardwareOverviewExportModel struct {
	DeviceName  string      `json:"deviceName"  dc:"机器名"`
	GroupName   string      `json:"groupName"   dc:"分组"`
	Motherboard string      `json:"motherboard" dc:"主板"`
	BiosVersion string      `json:"biosVersion" dc:"BIOS版本"`
	Cpu         string      `json:"cpu"         dc:"CPU"`
	Memory      string      `json:"memory"      dc:"内存"`
	Gpu         string      `json:"gpu"         dc:"显卡"`
	Nic         string      `json:"nic"         dc:"网卡"`
	ChangedAt   *gtime.Time `json:"changedAt"   dc:"变更时间"`
}
