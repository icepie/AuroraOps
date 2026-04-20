// ================================================================================
// Code generated and maintained by GoFrame CLI tool. DO NOT EDIT.
// You can delete these comments if you wish manually maintain this interface file.
// ================================================================================

package service

import (
	"context"
	"hotgo/api/servmsg"
	"hotgo/internal/library/cron"
	"hotgo/internal/library/network/tcp"
)

type (
	ITCPServer interface {
		// OnAuthSummary 获取授权信息
		OnAuthSummary(ctx context.Context, req *servmsg.AuthSummaryReq)
		// CronDelete 删除任务
		CronDelete(ctx context.Context, in *servmsg.CronDeleteReq) (err error)
		// CronEdit 编辑任务
		CronEdit(ctx context.Context, in *servmsg.CronEditReq) (err error)
		// CronStatus 修改任务状态
		CronStatus(ctx context.Context, in *servmsg.CronStatusReq) (err error)
		// CronOnlineExec 执行一次任务
		CronOnlineExec(ctx context.Context, in *servmsg.CronOnlineExecReq) (err error)
		// DispatchLog 查看调度日志
		DispatchLog(ctx context.Context, in *servmsg.CronDispatchLogReq) (log *cron.Log, err error)
		// OnExampleHello 一个tcp请求例子
		OnExampleHello(ctx context.Context, req *servmsg.ExampleHelloReq)
		// OnExampleRPCHello 一个rpc请求例子
		OnExampleRPCHello(ctx context.Context, req *servmsg.ExampleRPCHelloReq) (res *servmsg.ExampleRPCHelloRes, err error)
		// Instance 获取实例
		Instance() *tcp.Server
		// Start 启动服务
		Start(ctx context.Context)
		// Stop 关闭服务
		Stop(ctx context.Context)
		// DefaultInterceptor 默认拦截器
		DefaultInterceptor(ctx context.Context, msg *tcp.Message) (err error)
		// PreFilterInterceptor 预处理
		PreFilterInterceptor(ctx context.Context, msg *tcp.Message) (err error)
		// CreateTerminalSession 创建设备终端会话
		CreateTerminalSession(ctx context.Context, deviceId uint64, userId int64) (sessionId string, err error)
		// SendTerminalOpen 打开终端
		SendTerminalOpen(ctx context.Context, sessionId string, cols, rows uint32, shell string) (err error)
		// SendTerminalInput 发送终端输入
		SendTerminalInput(ctx context.Context, sessionId, input string) (err error)
		// SendTerminalResize 调整终端尺寸
		SendTerminalResize(ctx context.Context, sessionId string, cols, rows uint32) (err error)
		// SendTerminalClose 关闭终端
		SendTerminalClose(ctx context.Context, sessionId, message string) (err error)
		// SubscribeTerminal 订阅终端输出
		SubscribeTerminal(sessionId string) (ch <-chan []byte, cancel func(), err error)
	}
)

var (
	localTCPServer ITCPServer
)

func TCPServer() ITCPServer {
	if localTCPServer == nil {
		panic("implement not found for interface ITCPServer, forgot register?")
	}
	return localTCPServer
}

func RegisterTCPServer(i ITCPServer) {
	localTCPServer = i
}
