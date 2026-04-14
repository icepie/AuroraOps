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

type OpsDeviceGroupUpdateFields struct {
	Name   string `json:"name"   dc:"分组名称"`
	Sort   int    `json:"sort"   dc:"排序"`
	Remark string `json:"remark" dc:"备注"`
	Status int    `json:"status" dc:"状态"`
}

type OpsDeviceGroupInsertFields struct {
	Name   string `json:"name"   dc:"分组名称"`
	Sort   int    `json:"sort"   dc:"排序"`
	Remark string `json:"remark" dc:"备注"`
	Status int    `json:"status" dc:"状态"`
}

type OpsDeviceGroupEditInp struct {
	entity.OpsDeviceGroup
}

func (in *OpsDeviceGroupEditInp) Filter(ctx context.Context) (err error) {
	if verr := g.Validator().Rules("required").Data(in.Name).Messages("分组名称不能为空").Run(ctx); verr != nil {
		return verr.Current()
	}
	if verr := g.Validator().Rules("required").Data(in.Sort).Messages("排序不能为空").Run(ctx); verr != nil {
		return verr.Current()
	}
	return
}

type OpsDeviceGroupEditModel struct{}

type OpsDeviceGroupDeleteInp struct {
	Id interface{} `json:"id" v:"required#分组ID不能为空" dc:"分组ID"`
}

func (in *OpsDeviceGroupDeleteInp) Filter(ctx context.Context) (err error) {
	return
}

type OpsDeviceGroupDeleteModel struct{}

type OpsDeviceGroupViewInp struct {
	Id uint64 `json:"id" v:"required#分组ID不能为空" dc:"分组ID"`
}

func (in *OpsDeviceGroupViewInp) Filter(ctx context.Context) (err error) {
	return
}

type OpsDeviceGroupViewModel struct {
	entity.OpsDeviceGroup
}

type OpsDeviceGroupListInp struct {
	form.PageReq
	Name      string        `json:"name"      dc:"分组名称"`
	Status    int           `json:"status"    dc:"状态"`
	CreatedAt []*gtime.Time `json:"createdAt" dc:"创建时间"`
}

func (in *OpsDeviceGroupListInp) Filter(ctx context.Context) (err error) {
	return
}

type OpsDeviceGroupListModel struct {
	Id          uint64      `json:"id"          dc:"分组ID"`
	Name        string      `json:"name"        dc:"分组名称"`
	Sort        int         `json:"sort"        dc:"排序"`
	Remark      string      `json:"remark"      dc:"备注"`
	Status      int         `json:"status"      dc:"状态"`
	DeviceCount int         `json:"deviceCount" dc:"设备数量"`
	CreatedAt   *gtime.Time `json:"createdAt"   dc:"创建时间"`
}

type OpsDeviceGroupMaxSortInp struct{}

func (in *OpsDeviceGroupMaxSortInp) Filter(ctx context.Context) (err error) {
	return
}

type OpsDeviceGroupMaxSortModel struct {
	Sort int `json:"sort" dc:"排序"`
}

type OpsDeviceGroupStatusInp struct {
	Id     uint64 `json:"id"     v:"required#分组ID不能为空" dc:"分组ID"`
	Status int    `json:"status" dc:"状态"`
}

func (in *OpsDeviceGroupStatusInp) Filter(ctx context.Context) (err error) {
	if in.Id == 0 {
		return gerror.New("分组ID不能为空")
	}
	if in.Status <= 0 {
		return gerror.New("状态不能为空")
	}
	if !validate.InSlice(consts.StatusSlice, in.Status) {
		return gerror.New("状态不正确")
	}
	return
}

type OpsDeviceGroupStatusModel struct{}
