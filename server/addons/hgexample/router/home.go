// Package router
// @Link  https://github.com/bufanyun/hotgo
// @Copyright  Copyright (c) 2023 HotGo CLI
// @Author  Ms <133814250@qq.com>
// @License  https://github.com/bufanyun/hotgo/blob/master/LICENSE
package router

import (
	"auroraops/addons/hgexample/controller/home"
	"auroraops/addons/hgexample/global"
	"auroraops/internal/consts"
	"auroraops/internal/library/addons"
	"context"
	"github.com/gogf/gf/v2/net/ghttp"
)

// Home 前台页面路由
func Home(ctx context.Context, group *ghttp.RouterGroup) {
	prefix := addons.RouterPrefix(ctx, consts.AppHome, global.GetSkeleton().Name)
	group.Group(prefix, func(group *ghttp.RouterGroup) {
		group.Bind(
			home.Index,
		)
	})
}
