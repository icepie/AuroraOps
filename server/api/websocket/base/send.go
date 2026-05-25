// Package base
// @Link  https://github.com/bufanyun/hotgo
// @Copyright  Copyright (c) 2023 HotGo CLI
// @Author  Ms <133814250@qq.com>
// @License  https://github.com/bufanyun/hotgo/blob/master/LICENSE
package base

import (
	"auroraops/internal/model/input/websocketin"
	"github.com/gogf/gf/v2/frame/g"
)

// SendToTagReq 发送标签消息
type SendToTagReq struct {
	g.Meta `path:"/send/toTag" method:"post" tags:"WebSocket" summary:"发送标签消息"`
	websocketin.SendToTagInp
}

type SendToTagRes struct {
}
