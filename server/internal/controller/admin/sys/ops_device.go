package sys

import (
	"context"
	"encoding/json"
	"hotgo/api/admin/opsdevice"
	"hotgo/internal/model/input/sysin"
	"hotgo/internal/service"
	"net/http"

	"github.com/gogf/gf/v2/net/ghttp"
	"github.com/gorilla/websocket"
)

var (
	OpsDevice = cOpsDevice{}
)

type cOpsDevice struct{}

var terminalUpgrader = websocket.Upgrader{
	ReadBufferSize:  4096,
	WriteBufferSize: 4096,
	CheckOrigin: func(r *http.Request) bool {
		return true
	},
}

func (c *cOpsDevice) List(ctx context.Context, req *opsdevice.ListReq) (res *opsdevice.ListRes, err error) {
	list, totalCount, err := service.SysOpsDevice().List(ctx, &req.OpsDeviceListInp)
	if err != nil {
		return
	}

	if list == nil {
		list = []*sysin.OpsDeviceListModel{}
	}

	res = new(opsdevice.ListRes)
	res.List = list
	res.PageRes.Pack(req, totalCount)
	return
}

func (c *cOpsDevice) Edit(ctx context.Context, req *opsdevice.EditReq) (res *opsdevice.EditRes, err error) {
	err = service.SysOpsDevice().Edit(ctx, &req.OpsDeviceEditInp)
	return
}

func (c *cOpsDevice) MaxSort(ctx context.Context, req *opsdevice.MaxSortReq) (res *opsdevice.MaxSortRes, err error) {
	data, err := service.SysOpsDevice().MaxSort(ctx, &req.OpsDeviceMaxSortInp)
	if err != nil {
		return
	}

	res = new(opsdevice.MaxSortRes)
	res.OpsDeviceMaxSortModel = data
	return
}

func (c *cOpsDevice) View(ctx context.Context, req *opsdevice.ViewReq) (res *opsdevice.ViewRes, err error) {
	data, err := service.SysOpsDevice().View(ctx, &req.OpsDeviceViewInp)
	if err != nil {
		return
	}

	res = new(opsdevice.ViewRes)
	res.OpsDeviceViewModel = data
	return
}

func (c *cOpsDevice) Delete(ctx context.Context, req *opsdevice.DeleteReq) (res *opsdevice.DeleteRes, err error) {
	err = service.SysOpsDevice().Delete(ctx, &req.OpsDeviceDeleteInp)
	return
}

func (c *cOpsDevice) Status(ctx context.Context, req *opsdevice.StatusReq) (res *opsdevice.StatusRes, err error) {
	err = service.SysOpsDevice().Status(ctx, &req.OpsDeviceStatusInp)
	return
}

func (c *cOpsDevice) Option(ctx context.Context, req *opsdevice.OptionReq) (res opsdevice.OptionRes, err error) {
	data, err := service.SysOpsDevice().Option(ctx)
	if err != nil {
		return nil, err
	}
	return opsdevice.OptionRes(data), nil
}

func (c *cOpsDevice) TerminalCreate(ctx context.Context, req *opsdevice.TerminalCreateReq) (res *opsdevice.TerminalCreateRes, err error) {
	data, err := service.SysOpsDevice().CreateTerminalSession(ctx, &req.OpsDeviceTerminalCreateInp)
	if err != nil {
		return
	}
	res = new(opsdevice.TerminalCreateRes)
	res.OpsDeviceTerminalCreateModel = data
	return
}

func (c *cOpsDevice) TerminalWS(r *ghttp.Request) {
	sessionID := r.Get("sessionId").String()
	if sessionID == "" {
		r.Response.WriteStatusExit(http.StatusBadRequest, "sessionId is required")
		return
	}

	ch, cancel, err := service.TCPServer().SubscribeTerminal(sessionID)
	if err != nil {
		r.Response.WriteStatusExit(http.StatusBadRequest, err.Error())
		return
	}
	defer cancel()

	conn, err := terminalUpgrader.Upgrade(r.Response.Writer, r.Request, nil)
	if err != nil {
		return
	}
	defer conn.Close()

	go func() {
		for payload := range ch {
			if writeErr := conn.WriteMessage(websocket.TextMessage, payload); writeErr != nil {
				return
			}
		}
	}()

	ctx := r.Context()
	_ = service.TCPServer().SendTerminalOpen(ctx, sessionID, 120, 32, "")

	for {
		_, message, readErr := conn.ReadMessage()
		if readErr != nil {
			break
		}

		var payload struct {
			Type  string `json:"type"`
			Input string `json:"input"`
			Cols  uint32 `json:"cols"`
			Rows  uint32 `json:"rows"`
		}
		if err = json.Unmarshal(message, &payload); err != nil {
			continue
		}

		switch payload.Type {
		case "input":
			if payload.Input != "" {
				_ = service.TCPServer().SendTerminalInput(ctx, sessionID, payload.Input)
			}
		case "resize":
			_ = service.TCPServer().SendTerminalResize(ctx, sessionID, payload.Cols, payload.Rows)
		case "close":
			_ = service.TCPServer().SendTerminalClose(ctx, sessionID, "terminal closed by user")
		}
	}

	_ = service.TCPServer().SendTerminalClose(ctx, sessionID, "terminal disconnected")
}
