// Package router
// @Link  https://github.com/bufanyun/hotgo
// @Copyright  Copyright (c) 2023 HotGo CLI
// @Author  Ms <133814250@qq.com>
// @License  https://github.com/bufanyun/hotgo/blob/master/LICENSE
package router

import (
	"auroraops/addons/hgexample/controller/admin/sys"
	"auroraops/addons/hgexample/global"
	"auroraops/addons/hgexample/router/genrouter"
	"auroraops/internal/consts"
	"auroraops/internal/library/addons"
	"auroraops/internal/service"
	"context"
	"github.com/gogf/gf/v2/net/ghttp"
)

func Admin(ctx context.Context, group *ghttp.RouterGroup) {
	prefix := addons.RouterPrefix(ctx, consts.AppAdmin, global.GetSkeleton().Name)
	group.Group(prefix, func(group *ghttp.RouterGroup) {
		group.Bind(
			sys.Index,
		)
		group.Middleware(service.Middleware().AdminAuth)
		group.Bind(
			sys.Comp,
			sys.Config,
			sys.Table,
			sys.TreeTable,
		)
	})

	// 注册生成路由
	genrouter.Register(ctx, group)
}
