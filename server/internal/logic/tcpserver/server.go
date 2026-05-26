// Package tcpserver
// @Link  https://github.com/bufanyun/hotgo
// @Copyright  Copyright (c) 2023 HotGo CLI
// @Author  Ms <133814250@qq.com>
// @License  https://github.com/bufanyun/hotgo/blob/master/LICENSE
package tcpserver

import (
	"auroraops/internal/library/network/tcp"
	"auroraops/internal/service"
	"auroraops/utility/simple"
	"context"
	"github.com/gogf/gf/v2/frame/g"
)

type sTCPServer struct {
	serv        *tcp.Server
	terminals   *terminalManager
	terminalHub *terminalHub
	desktops    *desktopManager
	desktopHub  *desktopHub
}

func init() {
	service.RegisterTCPServer(newTCPServer())
}

func newTCPServer() *sTCPServer {
	return &sTCPServer{
		terminals:   newTerminalManager(),
		terminalHub: newTerminalHub(),
		desktops:    newDesktopManager(),
		desktopHub:  newDesktopHub(),
	}
}

// Instance 获取实例
func (s *sTCPServer) Instance() *tcp.Server {
	return s.serv
}

// Start 启动服务
func (s *sTCPServer) Start(ctx context.Context) {
	simple.SafeGo(ctx, func(ctx context.Context) {
		g.Log().Debug(ctx, "TCPServer start..")

		s.serv = tcp.NewServer(&tcp.ServerConfig{
			Name: simple.AppName(ctx),
			Addr: g.Cfg().MustGet(ctx, "tcp.server.address").String(),
		})

		// 注册路由
		s.serv.RegisterRouter(
			s.onServerLogin,     // 服务登录
			s.onServerHeartbeat, // 心跳
			s.onDeviceLogin,     // 设备登录
			s.onDeviceHeartbeat, // 设备心跳
			s.onDeviceMonitorReport,
			s.onDeviceTerminalOutput,
			s.onDeviceTerminalClosed,
			s.onDeviceDesktopTextOutput,
			s.onDeviceDesktopBinaryOutput,
			s.onDeviceDesktopClosed,
			s.OnAuthSummary,  // 获取授权信息
			s.OnExampleHello, // 一个tcp请求例子
		)

		// 注册RPC路由
		s.serv.RegisterRPCRouter(
			s.OnExampleRPCHello, // 一个rpc请求例子
		)

		// 注册拦截器
		s.serv.RegisterInterceptor(s.DefaultInterceptor, s.PreFilterInterceptor)

		go s.terminals.cleanupLoop(ctx)
		go s.desktops.cleanupLoop(ctx)

		// 服务监听
		if err := s.serv.Listen(); err != nil {
			if !s.serv.IsClose() {
				g.Log().Warningf(ctx, "TCPServer Listen err:%v", err)
			}
		}
	})
}

// Stop 关闭服务
func (s *sTCPServer) Stop(ctx context.Context) {
	if s.serv != nil && !s.serv.IsClose() {
		s.serv.Close()
		g.Log().Debug(ctx, "TCPServer stop..")
	}
}
