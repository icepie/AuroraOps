package sysin

import (
	"context"
	"hotgo/internal/consts"
	"hotgo/internal/model/entity"
	"hotgo/internal/model/input/form"
	"hotgo/utility/validate"

	"github.com/gogf/gf/v2/errors/gerror"
	"github.com/gogf/gf/v2/frame/g"
	"github.com/gogf/gf/v2/os/gtime"
)

type OpsDeviceUpdateFields struct {
	GroupId    uint64 `json:"groupId"    dc:"设备分组"`
	Name       string `json:"name"       dc:"设备名称"`
	Hostname   string `json:"hostname"   dc:"主机名"`
	Ip         string `json:"ip"         dc:"IP地址"`
	DeviceType string `json:"deviceType" dc:"设备类型"`
	OsName     string `json:"osName"     dc:"操作系统"`
	Location   string `json:"location"   dc:"部署位置"`
	Sort       int    `json:"sort"       dc:"排序"`
	Remark     string `json:"remark"     dc:"备注"`
	Status     int    `json:"status"     dc:"状态"`
}

type OpsDeviceInsertFields struct {
	GroupId    uint64 `json:"groupId"    dc:"设备分组"`
	Name       string `json:"name"       dc:"设备名称"`
	Hostname   string `json:"hostname"   dc:"主机名"`
	Ip         string `json:"ip"         dc:"IP地址"`
	DeviceType string `json:"deviceType" dc:"设备类型"`
	OsName     string `json:"osName"     dc:"操作系统"`
	Location   string `json:"location"   dc:"部署位置"`
	Sort       int    `json:"sort"       dc:"排序"`
	Remark     string `json:"remark"     dc:"备注"`
	Status     int    `json:"status"     dc:"状态"`
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
	DeviceType string        `json:"deviceType" dc:"设备类型"`
	Status     int           `json:"status"     dc:"状态"`
	CreatedAt  []*gtime.Time `json:"createdAt"  dc:"创建时间"`
}

func (in *OpsDeviceListInp) Filter(ctx context.Context) (err error) {
	return
}

type OpsDeviceListModel struct {
	Id         uint64      `json:"id"         dc:"设备ID"`
	GroupId    uint64      `json:"groupId"    dc:"分组ID"`
	GroupName  string      `json:"groupName"  dc:"设备分组"`
	Name       string      `json:"name"       dc:"设备名称"`
	Hostname   string      `json:"hostname"   dc:"主机名"`
	Ip         string      `json:"ip"         dc:"IP地址"`
	DeviceType string      `json:"deviceType" dc:"设备类型"`
	OsName     string      `json:"osName"     dc:"操作系统"`
	Location   string      `json:"location"   dc:"部署位置"`
	Status     int         `json:"status"     dc:"状态"`
	CreatedAt  *gtime.Time `json:"createdAt"  dc:"创建时间"`
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
	Name       string `json:"name"       dc:"设备名称"`
	Hostname   string `json:"hostname"   dc:"主机名"`
	Ip         string `json:"ip"         dc:"IP地址"`
	DeviceType string `json:"deviceType" dc:"设备类型"`
	OsName     string `json:"osName"     dc:"操作系统"`
	Location   string `json:"location"   dc:"部署位置"`
}

func (in *OpsDeviceClientRegisterInp) Filter(ctx context.Context) (err error) {
	if verr := g.Validator().Rules("required").Data(in.Name).Messages("设备名称不能为空").Run(ctx); verr != nil {
		return verr.Current()
	}
	if verr := g.Validator().Rules("required").Data(in.Hostname).Messages("主机名不能为空").Run(ctx); verr != nil {
		return verr.Current()
	}
	return
}

type OpsDeviceClientRegisterModel struct {
	Id         uint64 `json:"id"         dc:"设备ID"`
	Name       string `json:"name"       dc:"设备名称"`
	Hostname   string `json:"hostname"   dc:"主机名"`
	Ip         string `json:"ip"         dc:"IP地址"`
	Created    bool   `json:"created"    dc:"是否新建设备"`
	CreatedAt  string `json:"createdAt"  dc:"创建结果"`
	Token      string `json:"token"      dc:"设备接入令牌"`
	TcpAddress string `json:"tcpAddress" dc:"TCP服务地址"`
}

type OpsDeviceClientHeartbeatInp struct {
	Id       uint64 `json:"id"       dc:"设备ID"`
	Hostname string `json:"hostname" dc:"主机名"`
	Ip       string `json:"ip"       dc:"IP地址"`
	OsName   string `json:"osName"   dc:"操作系统"`
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
