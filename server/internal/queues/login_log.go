// Package queues
// @Link  https://github.com/bufanyun/hotgo
// @Copyright  Copyright (c) 2023 HotGo CLI
// @Author  Ms <133814250@qq.com>
// @License  https://github.com/bufanyun/hotgo/blob/master/LICENSE
package queues

import (
	"auroraops/internal/consts"
	"auroraops/internal/library/queue"
	"auroraops/internal/model/entity"
	"auroraops/internal/service"
	"context"
	"encoding/json"
)

func init() {
	queue.RegisterConsumer(LoginLog)
}

// LoginLog 登录日志
var LoginLog = &qLoginLog{}

type qLoginLog struct{}

// GetTopic 主题
func (q *qLoginLog) GetTopic() string {
	return consts.QueueLoginLogTopic
}

// Handle 处理消息
func (q *qLoginLog) Handle(ctx context.Context, mqMsg queue.MqMsg) (err error) {
	var data entity.SysLoginLog
	if err = json.Unmarshal(mqMsg.Body, &data); err != nil {
		return err
	}
	return service.SysLoginLog().RealWrite(ctx, data)
}
