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

type OpsDeviceUpdateFields struct {
	GroupId       uint64 `json:"groupId"    dc:"设备分组"`
	Name          string `json:"name"       dc:"设备名称"`
	Hostname      string `json:"hostname"   dc:"主机名"`
	Ip            string `json:"ip"         dc:"IP地址"`
	MacAddress    string `json:"macAddress" dc:"MAC地址"`
	DeviceType    string `json:"deviceType" dc:"设备类型"`
	OsName        string `json:"osName"     dc:"操作系统"`
	Architecture  string `json:"architecture" dc:"系统架构"`
	KernelVersion string `json:"kernelVersion" dc:"内核版本"`
	Location      string `json:"location"   dc:"部署位置"`
	Sort          int    `json:"sort"       dc:"排序"`
	Remark        string `json:"remark"     dc:"备注"`
	Status        int    `json:"status"     dc:"状态"`
}

type OpsDeviceInsertFields struct {
	GroupId       uint64 `json:"groupId"    dc:"设备分组"`
	Name          string `json:"name"       dc:"设备名称"`
	Hostname      string `json:"hostname"   dc:"主机名"`
	Ip            string `json:"ip"         dc:"IP地址"`
	MacAddress    string `json:"macAddress" dc:"MAC地址"`
	DeviceType    string `json:"deviceType" dc:"设备类型"`
	OsName        string `json:"osName"     dc:"操作系统"`
	Architecture  string `json:"architecture" dc:"系统架构"`
	KernelVersion string `json:"kernelVersion" dc:"内核版本"`
	Location      string `json:"location"   dc:"部署位置"`
	Sort          int    `json:"sort"       dc:"排序"`
	Remark        string `json:"remark"     dc:"备注"`
	Status        int    `json:"status"     dc:"状态"`
}

type OpsDeviceEditInp struct {
	entity.OpsDevice
}

func (in *OpsDeviceEditInp) Filter(ctx context.Context) (err error) {
	if verr := g.Validator().Rules("required").Data(in.Name).Messages("设备名称不能为空").Run(ctx); verr != nil {
		return verr.Current()
	}
	if verr := g.Validator().Rules("required").Data(in.Hostname).Messages("主机名不能为空").Run(ctx); verr != nil {
		return verr.Current()
	}
	if in.DeviceType != "" && in.DeviceType != "physical" && in.DeviceType != "virtual" {
		return gerror.New("设备类型只能为物理机或虚拟机")
	}
	if verr := g.Validator().Rules("required").Data(in.Sort).Messages("排序不能为空").Run(ctx); verr != nil {
		return verr.Current()
	}
	return
}

type OpsDeviceEditModel struct{}

type OpsDeviceDeleteInp struct {
	Id interface{} `json:"id" v:"required#设备ID不能为空" dc:"设备ID"`
}

func (in *OpsDeviceDeleteInp) Filter(ctx context.Context) (err error) {
	return
}

type OpsDeviceDeleteModel struct{}

type OpsDeviceViewInp struct {
	Id uint64 `json:"id" v:"required#设备ID不能为空" dc:"设备ID"`
}

func (in *OpsDeviceViewInp) Filter(ctx context.Context) (err error) {
	return
}

type OpsDeviceViewModel struct {
	entity.OpsDevice
}

type OpsDeviceListInp struct {
	form.PageReq
	Id         uint64        `json:"id"         dc:"设备ID"`
	GroupId    uint64        `json:"groupId"    dc:"分组ID"`
	GroupScope string        `json:"groupScope" dc:"分组筛选范围"`
	Name       string        `json:"name"       dc:"设备名称"`
	Hostname   string        `json:"hostname"   dc:"主机名"`
	Ip         string        `json:"ip"         dc:"IP地址"`
	MacAddress string        `json:"macAddress" dc:"MAC地址"`
	DeviceType string        `json:"deviceType" dc:"设备类型"`
	Status     int           `json:"status"     dc:"状态"`
	CreatedAt  []*gtime.Time `json:"createdAt"  dc:"创建时间"`
}

func (in *OpsDeviceListInp) Filter(ctx context.Context) (err error) {
	return
}

type OpsDeviceListModel struct {
	Id                uint64                `json:"id"         dc:"设备ID"`
	GroupId           uint64                `json:"groupId"    dc:"分组ID"`
	GroupName         string                `json:"groupName"  dc:"设备分组"`
	Name              string                `json:"name"       dc:"设备名称"`
	Hostname          string                `json:"hostname"   dc:"主机名"`
	Ip                string                `json:"ip"         dc:"IP地址"`
	MacAddress        string                `json:"macAddress" dc:"MAC地址"`
	DeviceType        string                `json:"deviceType" dc:"设备类型"`
	OsName            string                `json:"osName"     dc:"操作系统"`
	Architecture      string                `json:"architecture" dc:"系统架构"`
	KernelVersion     string                `json:"kernelVersion" dc:"内核版本"`
	Location          string                `json:"location"   dc:"部署位置"`
	MonitorSnapshot   string                `json:"-" dc:"监视快照"`
	Monitor           *OpsDeviceMonitorView `json:"monitor" dc:"监视快照"`
	MonitorReportedAt *gtime.Time           `json:"monitorReportedAt" dc:"监视上报时间"`
	Online            bool                  `json:"online"     dc:"在线状态"`
	Status            int                   `json:"status"     dc:"状态"`
	CreatedAt         *gtime.Time           `json:"createdAt"  dc:"创建时间"`
}

type OpsDeviceMonitorView struct {
	System             string                     `json:"system"             dc:"系统名称"`
	Architecture       string                     `json:"architecture"       dc:"系统架构"`
	KernelVersion      string                     `json:"kernelVersion"      dc:"内核版本"`
	CpuModel           string                     `json:"cpuModel"           dc:"CPU型号"`
	GpuModels          []string                   `json:"gpuModels"          dc:"GPU型号列表"`
	CpuPercent         float64                    `json:"cpuPercent"         dc:"CPU使用率"`
	MemoryPercent      float64                    `json:"memoryPercent"      dc:"内存使用率"`
	SwapPercent        float64                    `json:"swapPercent"        dc:"交换分区使用率"`
	SwapEnabled        bool                       `json:"swapEnabled"        dc:"是否启用交换分区"`
	DiskPercent        float64                    `json:"diskPercent"        dc:"磁盘使用率"`
	NetRxRateBytes     float64                    `json:"netRxRateBytes"     dc:"网络下行速率"`
	NetTxRateBytes     float64                    `json:"netTxRateBytes"     dc:"网络上行速率"`
	NetRxBytes         uint64                     `json:"netRxBytes"         dc:"网络累计下行流量"`
	NetTxBytes         uint64                     `json:"netTxBytes"         dc:"网络累计上行流量"`
	CpuCores           int                        `json:"cpuCores"           dc:"CPU核心数"`
	CpuPhysicalCores   int                        `json:"cpuPhysicalCores"   dc:"CPU物理核心数"`
	MemoryUsedBytes    uint64                     `json:"memoryUsedBytes"    dc:"内存已用量"`
	MemoryTotalBytes   uint64                     `json:"memoryTotalBytes"   dc:"内存总量"`
	SwapUsedBytes      uint64                     `json:"swapUsedBytes"      dc:"交换分区已用量"`
	SwapTotalBytes     uint64                     `json:"swapTotalBytes"     dc:"交换分区总量"`
	DiskUsedBytes      uint64                     `json:"diskUsedBytes"      dc:"磁盘已用量"`
	DiskTotalBytes     uint64                     `json:"diskTotalBytes"     dc:"磁盘总量"`
	Load1              float64                    `json:"load1"              dc:"1分钟负载"`
	Load5              float64                    `json:"load5"              dc:"5分钟负载"`
	Load15             float64                    `json:"load15"             dc:"15分钟负载"`
	ProcessCount       int                        `json:"processCount"       dc:"进程数"`
	TcpConnectionCount int                        `json:"tcpConnectionCount" dc:"TCP连接数"`
	UdpConnectionCount int                        `json:"udpConnectionCount" dc:"UDP连接数"`
	Temperatures       []OpsDeviceTemperatureView `json:"temperatures"       dc:"温度列表"`
	BootTimeSeconds    uint64                     `json:"bootTimeSeconds"    dc:"系统启动时间"`
	UptimeSeconds      uint64                     `json:"uptimeSeconds"      dc:"在线时长"`
	AgentVersion       string                     `json:"agentVersion"       dc:"Agent版本"`
}

type OpsDeviceTemperatureView struct {
	Name     string   `json:"name"     dc:"传感器名称"`
	Value    float64  `json:"value"    dc:"当前温度"`
	Kind     string   `json:"kind"     dc:"传感器类型"`
	Max      *float64 `json:"max"      dc:"最高温度"`
	Critical *float64 `json:"critical" dc:"临界温度"`
}

type OpsDeviceMaxSortInp struct{}

func (in *OpsDeviceMaxSortInp) Filter(ctx context.Context) (err error) {
	return
}

type OpsDeviceMaxSortModel struct {
	Sort int `json:"sort" dc:"排序"`
}

type OpsDeviceStatusInp struct {
	Id     uint64 `json:"id"     v:"required#设备ID不能为空" dc:"设备ID"`
	Status int    `json:"status" dc:"状态"`
}

func (in *OpsDeviceStatusInp) Filter(ctx context.Context) (err error) {
	if in.Id == 0 {
		return gerror.New("设备ID不能为空")
	}
	if in.Status <= 0 {
		return gerror.New("状态不能为空")
	}
	if !validate.InSlice(consts.StatusSlice, in.Status) {
		return gerror.New("状态不正确")
	}
	return
}

type OpsDeviceStatusModel struct{}

type OpsDeviceClientRegisterInp struct {
	Name          string `json:"name"       dc:"设备名称"`
	Hostname      string `json:"hostname"   dc:"主机名"`
	Ip            string `json:"ip"         dc:"IP地址"`
	MacAddress    string `json:"macAddress" dc:"MAC地址"`
	DeviceType    string `json:"deviceType" dc:"设备类型"`
	OsName        string `json:"osName"     dc:"操作系统"`
	Architecture  string `json:"architecture" dc:"系统架构"`
	KernelVersion string `json:"kernelVersion" dc:"内核版本"`
	Location      string `json:"location"   dc:"部署位置"`
}

func (in *OpsDeviceClientRegisterInp) Filter(ctx context.Context) (err error) {
	if verr := g.Validator().Rules("required").Data(in.Name).Messages("设备名称不能为空").Run(ctx); verr != nil {
		return verr.Current()
	}
	if verr := g.Validator().Rules("required").Data(in.Hostname).Messages("主机名不能为空").Run(ctx); verr != nil {
		return verr.Current()
	}
	if in.DeviceType != "" && in.DeviceType != "physical" && in.DeviceType != "virtual" {
		return gerror.New("设备类型只能为物理机或虚拟机")
	}
	return
}

type OpsDeviceClientRegisterModel struct {
	Id         uint64 `json:"id"         dc:"设备ID"`
	Name       string `json:"name"       dc:"设备名称"`
	Hostname   string `json:"hostname"   dc:"主机名"`
	Ip         string `json:"ip"         dc:"IP地址"`
	MacAddress string `json:"macAddress" dc:"MAC地址"`
	Created    bool   `json:"created"    dc:"是否新建设备"`
	CreatedAt  string `json:"createdAt"  dc:"创建结果"`
	Token      string `json:"token"      dc:"设备接入令牌"`
	TcpAddress string `json:"tcpAddress" dc:"TCP服务地址"`
}

type OpsDeviceClientHeartbeatInp struct {
	Id            uint64 `json:"id"       dc:"设备ID"`
	Hostname      string `json:"hostname" dc:"主机名"`
	Ip            string `json:"ip"       dc:"IP地址"`
	MacAddress    string `json:"macAddress" dc:"MAC地址"`
	OsName        string `json:"osName"   dc:"操作系统"`
	Architecture  string `json:"architecture" dc:"系统架构"`
	KernelVersion string `json:"kernelVersion" dc:"内核版本"`
}

func (in *OpsDeviceClientHeartbeatInp) Filter(ctx context.Context) (err error) {
	if in.Id == 0 {
		return gerror.New("设备ID不能为空")
	}
	if verr := g.Validator().Rules("required").Data(in.Hostname).Messages("主机名不能为空").Run(ctx); verr != nil {
		return verr.Current()
	}
	return
}

type OpsDeviceClientHeartbeatModel struct {
	Id      uint64 `json:"id"      dc:"设备ID"`
	AliveAt string `json:"aliveAt" dc:"心跳时间"`
}

type OpsDeviceTcpLoginModel struct {
	DeviceId    uint64 `json:"deviceId"   dc:"设备ID"`
	Name        string `json:"name"       dc:"设备名称"`
	Hostname    string `json:"hostname"   dc:"主机名"`
	ConnectedAt string `json:"connectedAt" dc:"连接时间"`
}

type OpsDeviceWakeInp struct {
	Id         uint64 `json:"id"         v:"required#设备ID不能为空" dc:"设备ID"`
	MacAddress string `json:"macAddress" dc:"MAC地址"`
	Broadcast  string `json:"broadcast"  dc:"广播地址"`
	Port       int    `json:"port"       dc:"UDP端口"`
	Repeat     int    `json:"repeat"     dc:"重复发送次数"`
}

func (in *OpsDeviceWakeInp) Filter(ctx context.Context) (err error) {
	if in.Id == 0 {
		return gerror.New("设备ID不能为空")
	}
	if in.Port < 0 || in.Port > 65535 {
		return gerror.New("WOL端口不正确")
	}
	if in.Repeat < 0 || in.Repeat > 10 {
		return gerror.New("重复次数不能超过10次")
	}
	return
}

type OpsDeviceWakeModel struct {
	Id         uint64   `json:"id"         dc:"设备ID"`
	MacAddress string   `json:"macAddress" dc:"MAC地址"`
	Targets    []string `json:"targets"    dc:"发送目标"`
	Packets    int      `json:"packets"    dc:"发送包数"`
}
