// Package router
// @Link  https://github.com/bufanyun/hotgo
// @Copyright  Copyright (c) 2023 HotGo CLI
// @Author  Ms <133814250@qq.com>
// @License  https://github.com/bufanyun/hotgo/blob/master/LICENSE
package router

import (
	"auroraops/addons/hgexample/controller/websocket"
	"auroraops/addons/hgexample/controller/websocket/handler"
	"auroraops/addons/hgexample/global"
	"auroraops/internal/consts"
	"auroraops/internal/library/addons"
	"auroraops/internal/service"
	ws "auroraops/internal/websocket"
	"context"
	"github.com/gogf/gf/v2/net/ghttp"
)

// WebSocket ws路由配置
func WebSocket(ctx context.Context, group *ghttp.RouterGroup) {
	prefix := addons.RouterPrefix(ctx, consts.AppWebSocket, global.GetSkeleton().Name)
	group.Group(prefix, func(group *ghttp.RouterGroup) {
		group.Bind(
			// 无需验证的路由
			websocket.Index,
		)
		// ws连接中间件
		group.Middleware(service.Middleware().WebSocketAuth)
		group.Bind(
		// 需要验证的路由
		// ..
		)
	})

	// 注册消息路由
	ws.RegisterMsg(ws.EventHandlers{
		"admin/addons/hgexample/testMessage": handler.Index.TestMessage, // 测试消息
	})
}
