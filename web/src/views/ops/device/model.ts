import { h, ref } from 'vue';
import { cloneDeep } from 'lodash-es';
import { NIcon, NProgress, NSpace, NTag } from 'naive-ui';
import { BarChartOutlined } from '@vicons/antd';
import { SocketEnum } from '@/enums/socketEnum';
import { FormSchema } from '@/components/Form';
import { defRangeShortcuts } from '@/utils/dateUtil';
import { renderOptionTag } from '@/utils';
import { useDictStore } from '@/store/modules/dict';
import { Option as DeviceGroupOption } from '@/api/opsDeviceGroup';

const dict = useDictStore();
export const OPS_DEVICE_GROUP_OPTION_KEY = 'opsDeviceGroupOptionRemote';

export const DEVICE_MONITOR_TAG = 'ops:device:monitor';
export const DEVICE_MONITOR_EVENT = SocketEnum.EventOpsDeviceMonitor;

export interface DeviceMonitor {
  system?: string;
  architecture?: string;
  cpuModel?: string;
  gpuModels?: string[];
  cpuPercent?: number;
  memoryPercent?: number;
  swapPercent?: number;
  swapEnabled?: boolean;
  diskPercent?: number;
  netRxRateBytes?: number;
  netTxRateBytes?: number;
  netRxBytes?: number;
  netTxBytes?: number;
  cpuCores?: number;
  cpuPhysicalCores?: number;
  memoryUsedBytes?: number;
  memoryTotalBytes?: number;
  swapUsedBytes?: number;
  swapTotalBytes?: number;
  diskUsedBytes?: number;
  diskTotalBytes?: number;
  load1?: number;
  load5?: number;
  load15?: number;
  processCount?: number;
  tcpConnectionCount?: number;
  udpConnectionCount?: number;
  temperatures?: DeviceTemperature[];
  bootTimeSeconds?: number;
  uptimeSeconds?: number;
  agentVersion?: string;
}

export interface DeviceTemperature {
  name?: string;
  value?: number;
  kind?: string;
  max?: number | null;
  critical?: number | null;
}

export const deviceTypeOptions = [
  { label: '物理机', value: 'physical', key: 'physical' },
  { label: '虚拟机', value: 'virtual', key: 'virtual' },
];

export class State {
  public id = 0;
  public groupId = null;
  public groupName = '';
  public name = '';
  public hostname = '';
  public ip = '';
  public deviceType = 'physical';
  public osName = '';
  public architecture = '';
  public location = '';
  public monitor: DeviceMonitor | null = null;
  public monitorReportedAt = '';
  public online = false;
  public sort = 0;
  public status = 1;
  public remark = '';
  public createdAt = '';
  public updatedAt = '';
  public deletedAt = '';

  constructor(state?: Partial<State>) {
    if (state) {
      Object.assign(this, state);
    }
  }
}

export function newState(state: State | Record<string, any> | null): State {
  if (state !== null) {
    if (state instanceof State) {
      return cloneDeep(state);
    }
    return new State(state);
  }
  return new State();
}

export const rules = {
  name: {
    required: true,
    trigger: ['blur', 'input'],
    message: '请输入设备名称',
  },
  hostname: {
    required: true,
    trigger: ['blur', 'input'],
    message: '请输入主机名',
  },
  sort: {
    required: true,
    trigger: ['blur', 'input'],
    type: 'number',
    message: '请输入排序',
  },
};

export const schemas = ref<FormSchema[]>([
  {
    field: 'id',
    component: 'NInputNumber',
    label: '设备ID',
    componentProps: {
      placeholder: '请输入设备ID',
    },
  },
  {
    field: 'name',
    component: 'NInput',
    label: '设备名称',
    componentProps: {
      placeholder: '请输入设备名称',
    },
  },
  {
    field: 'hostname',
    component: 'NInput',
    label: '主机名',
    componentProps: {
      placeholder: '请输入主机名',
    },
  },
  {
    field: 'ip',
    component: 'NInput',
    label: 'IP地址',
    componentProps: {
      placeholder: '请输入IP地址',
    },
  },
  {
    field: 'deviceType',
    component: 'NSelect',
    label: '设备类型',
    defaultValue: null,
    componentProps: {
      placeholder: '请选择设备类型',
      options: deviceTypeOptions,
    },
  },
  {
    field: 'status',
    component: 'NSelect',
    label: '状态',
    defaultValue: null,
    componentProps: {
      placeholder: '请选择状态',
      options: dict.getOption('sys_normal_disable'),
    },
  },
  {
    field: 'createdAt',
    component: 'NDatePicker',
    label: '创建时间',
    componentProps: {
      type: 'datetimerange',
      clearable: true,
      shortcuts: defRangeShortcuts(),
    },
  },
]);

function getDeviceTypeLabel(value: string) {
  return deviceTypeOptions.find((item) => item.value === value)?.label || value || '-';
}

function getArchitectureLabel(row: State) {
  const architecture = String(row.architecture || '').trim();
  if (architecture) return architecture;
  const location = String(row.location || '').trim();
  if (/^(aarch64|arm64|amd64|x86_64|i386|i686)$/i.test(location)) return location;
  return '-';
}

function getLocationLabel(value: string) {
  const text = String(value || '').trim();
  if (!text) return '-';
  if (/^(aarch64|arm64|amd64|x86_64|i386|i686)$/i.test(text)) {
    return '-';
  }
  return text;
}

function percentValue(value?: number) {
  const num = Number(value || 0);
  if (!Number.isFinite(num)) return 0;
  return Math.max(0, Math.min(100, Math.round(num)));
}

function percentText(value?: number) {
  const num = Number(value || 0);
  if (!Number.isFinite(num)) return '0%';
  return `${Math.round(num)}%`;
}

function formatBytes(value?: number) {
  let size = Number(value || 0);
  if (!Number.isFinite(size) || size <= 0) return '0B';
  const units = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
  let index = 0;
  while (size >= 1024 && index < units.length - 1) {
    size /= 1024;
    index += 1;
  }
  const fixed = size >= 100 || index === 0 ? 0 : size >= 10 ? 1 : 2;
  return `${size.toFixed(fixed)}${units[index]}`;
}

function formatRate(value?: number) {
  return `${formatBytes(value)}/s`;
}

function formatDuration(seconds?: number) {
  let total = Math.max(0, Math.floor(Number(seconds || 0)));
  const days = Math.floor(total / 86400);
  total %= 86400;
  const hours = Math.floor(total / 3600);
  total %= 3600;
  const minutes = Math.floor(total / 60);
  if (days > 0) return `${days}天 ${hours}小时`;
  if (hours > 0) return `${hours}小时 ${minutes}分`;
  return `${minutes}分`;
}

function formatTimestamp(seconds?: number) {
  const value = Number(seconds || 0);
  if (!Number.isFinite(value) || value <= 0) return '-';
  const date = new Date(value * 1000);
  const pad = (num: number) => String(num).padStart(2, '0');
  return `${date.getFullYear()}/${pad(date.getMonth() + 1)}/${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`;
}

function formatBytePair(used?: number, total?: number) {
  return `${formatBytes(used)} / ${formatBytes(total)}`;
}

function formatTemperature(value?: number | null) {
  const num = Number(value);
  if (!Number.isFinite(num)) return '-';
  return `${num.toFixed(1)}°C`;
}

function normalizeTemperatureName(value?: string) {
  return String(value || '')
    .replace(/\s+/g, ' ')
    .trim();
}

function isFiniteTemperature(value?: number | null) {
  const num = Number(value);
  return Number.isFinite(num);
}

function isUsefulTemperatureLimit(value?: number | null) {
  const num = Number(value);
  return Number.isFinite(num) && num > 0;
}

function getTemperatureValue(item: DeviceTemperature) {
  return Number(item.value);
}

function isTemperatureKind(item: DeviceTemperature, kind: string) {
  return String(item.kind || '').toLowerCase() === kind;
}

function pickHottestTemperature(items: DeviceTemperature[]) {
  return items
    .filter((item) => isFiniteTemperature(item.value))
    .slice()
    .sort((a, b) => getTemperatureValue(b) - getTemperatureValue(a))[0];
}

function formatTemperatureLimitText(item?: DeviceTemperature) {
  if (!item) return '';
  const parts = [
    isUsefulTemperatureLimit(item.max) ? `最高 ${formatTemperature(item.max)}` : '',
    isUsefulTemperatureLimit(item.critical) ? `临界 ${formatTemperature(item.critical)}` : '',
  ].filter(Boolean);
  return parts.length ? ` (${parts.join(' / ')})` : '';
}

function shortenTemperatureName(name?: string) {
  const value = normalizeTemperatureName(name)
    .replace(/^coretemp\s+/i, '')
    .replace(/^nvme\s+/i, '')
    .replace(/^amdgpu\s+/i, 'AMD GPU ')
    .replace(/^nct6687\s+/i, '');
  return value || '传感器';
}

function formatTemperatureRows(monitor: DeviceMonitor) {
  const temperatures = (Array.isArray(monitor.temperatures) ? monitor.temperatures : []).filter(
    (item) => isFiniteTemperature(item?.value)
  );
  if (!temperatures.length) return [{ label: '传感器', value: '暂无' }];

  const rows: Array<{ label: string; value: string }> = [];
  const cpuReadings = temperatures.filter((item) => isTemperatureKind(item, 'cpu'));
  const gpuReadings = temperatures.filter((item) => isTemperatureKind(item, 'gpu'));
  const diskReadings = temperatures.filter((item) => isTemperatureKind(item, 'disk'));
  const boardReadings = temperatures.filter((item) => isTemperatureKind(item, 'board'));
  const sensorReadings = temperatures.filter((item) => isTemperatureKind(item, 'sensor'));

  const cpuPackage =
    cpuReadings.find((item) => /package|tctl|tdie/i.test(normalizeTemperatureName(item.name))) ||
    pickHottestTemperature(cpuReadings);
  const cpuHottest = pickHottestTemperature(cpuReadings);
  if (cpuPackage) {
    const parts = [`封装 ${formatTemperature(cpuPackage.value)}`];
    if (
      cpuHottest &&
      normalizeTemperatureName(cpuHottest.name) !== normalizeTemperatureName(cpuPackage.name)
    ) {
      parts.push(`最高核心 ${formatTemperature(cpuHottest.value)}`);
    }
    rows.push({
      label: 'CPU',
      value: `${parts.join(' / ')}${formatTemperatureLimitText(cpuPackage)}`,
    });
  }

  gpuReadings
    .slice()
    .sort((a, b) => getTemperatureValue(b) - getTemperatureValue(a))
    .slice(0, 2)
    .forEach((item) => {
      rows.push({
        label: 'GPU',
        value: `${shortenTemperatureName(item.name)} ${formatTemperature(item.value)}${formatTemperatureLimitText(item)}`,
      });
    });

  diskReadings
    .filter((item) => /composite|sensor\s*1|temp/i.test(normalizeTemperatureName(item.name)))
    .slice()
    .sort((a, b) => getTemperatureValue(b) - getTemperatureValue(a))
    .slice(0, 3)
    .forEach((item) => {
      rows.push({
        label: '磁盘',
        value: `${shortenTemperatureName(item.name)} ${formatTemperature(item.value)}${formatTemperatureLimitText(item)}`,
      });
    });

  const board = pickHottestTemperature(boardReadings);
  if (board) {
    rows.push({
      label: '主板',
      value: `${shortenTemperatureName(board.name)} ${formatTemperature(board.value)}${formatTemperatureLimitText(board)}`,
    });
  }

  const other = sensorReadings
    .filter((item) => !/core|package|nvme|amdgpu/i.test(normalizeTemperatureName(item.name)))
    .slice()
    .sort((a, b) => getTemperatureValue(b) - getTemperatureValue(a))[0];
  if (other && rows.length < 8) {
    rows.push({
      label: '其它',
      value: `${shortenTemperatureName(other.name)} ${formatTemperature(other.value)}${formatTemperatureLimitText(other)}`,
    });
  }

  return rows.length ? rows.slice(0, 8) : [{ label: '传感器', value: '暂无' }];
}

function formatCpuDetail(monitor: DeviceMonitor) {
  const model = String(monitor.cpuModel || '').trim() || 'CPU';
  const physical = Number(monitor.cpuPhysicalCores || 0);
  const logical = Number(monitor.cpuCores || 0);
  if (physical > 0 && logical > 0) {
    return `${model} ${physical} Physical Core / ${logical} Logical Core`;
  }
  if (logical > 0) return `${model} ${logical} Core`;
  return model;
}

function formatGpuDetail(monitor: DeviceMonitor) {
  const list = Array.isArray(monitor.gpuModels)
    ? monitor.gpuModels.filter((item) => String(item || '').trim())
    : [];
  return list.length ? list.join(' / ') : '-';
}

export const columns = [
  {
    type: 'expand',
    width: 48,
    expandable: () => true,
    renderExpand(row: State) {
      const monitor = row.monitor;
      if (!monitor) {
        return h('div', { class: 'device-monitor-empty' }, '暂无监视数据，等待客户端上报');
      }
      const metrics = [
        {
          label: 'CPU',
          value: percentText(monitor.cpuPercent),
          percent: percentValue(monitor.cpuPercent),
          type: 'info',
        },
        {
          label: '内存',
          value: percentText(monitor.memoryPercent),
          percent: percentValue(monitor.memoryPercent),
          type: percentValue(monitor.memoryPercent) >= 90 ? 'error' : 'success',
        },
        {
          label: '交换',
          value: monitor.swapEnabled ? percentText(monitor.swapPercent) : 'OFF',
          percent: monitor.swapEnabled ? percentValue(monitor.swapPercent) : 0,
          type: 'warning',
        },
        {
          label: '硬盘',
          value: percentText(monitor.diskPercent),
          percent: percentValue(monitor.diskPercent),
          type: percentValue(monitor.diskPercent) >= 90 ? 'error' : 'info',
        },
      ];
      const summaryRows = [
        {
          label: '网速',
          value: `${formatRate(monitor.netRxRateBytes)} / ${formatRate(monitor.netTxRateBytes)}`,
        },
        {
          label: '流量',
          value: `${formatBytes(monitor.netRxBytes)} / ${formatBytes(monitor.netTxBytes)}`,
        },
        { label: '在线', value: formatDuration(monitor.uptimeSeconds) },
        {
          label: '负载',
          value: `${Number(monitor.load1 || 0).toFixed(2)} / ${Number(monitor.load5 || 0).toFixed(2)} / ${Number(monitor.load15 || 0).toFixed(2)}`,
        },
      ];
      const detailGroups = [
        {
          title: '系统',
          rows: [
            {
              label: '系统',
              value: `${monitor.system || row.osName || '-'} - [${monitor.architecture || getArchitectureLabel(row)}]`,
            },
            { label: 'CPU', value: formatCpuDetail(monitor) },
            { label: 'GPU', value: formatGpuDetail(monitor) },
          ],
        },
        {
          title: '资源',
          rows: [
            { label: '硬盘', value: formatBytePair(monitor.diskUsedBytes, monitor.diskTotalBytes) },
            {
              label: '内存',
              value: formatBytePair(monitor.memoryUsedBytes, monitor.memoryTotalBytes),
            },
            {
              label: '交换',
              value: monitor.swapEnabled
                ? formatBytePair(monitor.swapUsedBytes, monitor.swapTotalBytes)
                : 'OFF',
            },
          ],
        },
        {
          title: '运行',
          rows: [
            { label: '进程数', value: String(monitor.processCount ?? '-') },
            {
              label: '连接数',
              value: `TCP ${monitor.tcpConnectionCount ?? 0} / UDP ${monitor.udpConnectionCount ?? 0}`,
            },
            { label: '启动', value: formatTimestamp(monitor.bootTimeSeconds) },
            { label: '活动', value: row.monitorReportedAt || '-' },
            { label: '版本', value: monitor.agentVersion || '-' },
          ],
        },
        {
          title: '温度',
          rows: formatTemperatureRows(monitor),
        },
      ];

      return h('div', { class: 'device-monitor-panel' }, [
        h('div', { class: 'device-monitor-panel__head' }, [
          h(
            NSpace,
            { align: 'center', size: 8 },
            {
              default: () => [
                h(NIcon, { size: 16 }, { default: () => h(BarChartOutlined) }),
                h('span', { class: 'device-monitor-panel__title' }, '监视'),
                h('span', { class: 'device-monitor-panel__time' }, row.monitorReportedAt || ''),
              ],
            }
          ),
        ]),
        h('div', { class: 'device-monitor-summary' }, [
          h(
            'div',
            { class: 'device-monitor-bars' },
            metrics.map((item) =>
              h('div', { class: 'device-monitor-bar' }, [
                h('div', { class: 'device-monitor-bar__label' }, item.label),
                h(
                  NProgress,
                  {
                    type: 'line',
                    percentage: item.percent,
                    indicatorPlacement: 'inside',
                    processing: false,
                    status: item.type as any,
                  },
                  { default: () => item.value }
                ),
              ])
            )
          ),
          h(
            'div',
            { class: 'device-monitor-facts' },
            summaryRows.map((item) =>
              h('div', { class: 'device-monitor-fact' }, [
                h('span', item.label),
                h('b', item.value),
              ])
            )
          ),
        ]),
        h(
          'div',
          { class: 'device-monitor-detail' },
          detailGroups.map((group) =>
            h(
              'div',
              {
                class: [
                  'device-monitor-detail__group',
                  group.title === '温度' ? 'device-monitor-detail__temperature' : '',
                ],
              },
              [
                h('div', { class: 'device-monitor-detail__title' }, group.title),
                ...group.rows.map((item) =>
                  h('div', { class: 'device-monitor-detail__row' }, [
                    h('span', { class: 'device-monitor-detail__label' }, item.label),
                    h('b', { class: 'device-monitor-detail__value' }, item.value),
                  ])
                ),
              ]
            )
          )
        ),
      ]);
    },
  },
  {
    title: '设备ID',
    key: 'id',
    align: 'left',
    width: 90,
  },
  {
    title: '在线状态',
    key: 'online',
    align: 'left',
    width: 100,
    render(row: State) {
      return h(
        NTag,
        {
          type: row.online ? 'success' : 'default',
          bordered: false,
        },
        {
          default: () => (row.online ? '在线' : '离线'),
        }
      );
    },
  },
  {
    title: '设备分组',
    key: 'groupName',
    align: 'left',
    width: 140,
    render(row: State) {
      return h('span', {}, row.groupName || '未分组');
    },
  },
  {
    title: '设备名称',
    key: 'name',
    align: 'left',
    width: 180,
  },
  {
    title: '主机名',
    key: 'hostname',
    align: 'left',
    width: 180,
  },
  {
    title: 'IP地址',
    key: 'ip',
    align: 'left',
    width: 160,
  },
  {
    title: '设备类型',
    key: 'deviceType',
    align: 'left',
    width: 120,
    render(row: State) {
      return h('span', {}, getDeviceTypeLabel(row.deviceType));
    },
  },
  {
    title: '操作系统',
    key: 'osName',
    align: 'left',
    width: 220,
  },
  {
    title: '系统架构',
    key: 'architecture',
    align: 'left',
    width: 120,
    render(row: State) {
      return h('span', {}, getArchitectureLabel(row));
    },
  },
  {
    title: '部署位置',
    key: 'location',
    align: 'left',
    width: 180,
    render(row: State) {
      return h('span', {}, getLocationLabel(row.location));
    },
  },
  {
    title: '状态',
    key: 'status',
    align: 'left',
    width: 100,
    render(row: State) {
      return renderOptionTag('sys_normal_disable', row.status);
    },
  },
  {
    title: '创建时间',
    key: 'createdAt',
    align: 'left',
    width: 180,
  },
];

export function loadOptions() {
  dict.loadOptions(['sys_normal_disable']);
}

export async function loadGroupOptions() {
  const options = await DeviceGroupOption();
  dict.setOption(OPS_DEVICE_GROUP_OPTION_KEY, [
    {
      label: '未分组',
      value: 0,
      key: 0,
    },
    ...(options || []),
  ]);
}
