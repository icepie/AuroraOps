export enum SocketEnum {
  EventPing = 'ping',
  EventKick = 'kick',
  EventNotice = 'notice',
  EventConnected = 'connected',
  EventAdminMonitorTrends = 'admin/monitor/trends',
  EventAdminMonitorRunInfo = 'admin/monitor/runInfo',
  EventOpsDeviceMonitor = 'ops/device/monitor',
  EventAdminOrderNotify = 'admin/order/notify',
  HeartBeatInterval = 1000,
  CodeSuc = 0,
  CodeErr = -1,
}
