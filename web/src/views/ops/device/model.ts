import { h, ref } from 'vue';
import { cloneDeep } from 'lodash-es';
import { NTag } from 'naive-ui';
import { FormSchema } from '@/components/Form';
import { defRangeShortcuts } from '@/utils/dateUtil';
import { renderOptionTag } from '@/utils';
import { useDictStore } from '@/store/modules/dict';
import { Option as DeviceGroupOption } from '@/api/opsDeviceGroup';

const dict = useDictStore();
export const OPS_DEVICE_GROUP_OPTION_KEY = 'opsDeviceGroupOptionRemote';

export const deviceTypeOptions = [
  { label: '物理机', value: 'physical', key: 'physical' },
  { label: '虚拟机', value: 'virtual', key: 'virtual' },
  { label: '交换机', value: 'switch', key: 'switch' },
  { label: '路由器', value: 'router', key: 'router' },
  { label: '防火墙', value: 'firewall', key: 'firewall' },
  { label: '存储设备', value: 'storage', key: 'storage' },
  { label: '其他', value: 'other', key: 'other' },
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
  public location = '';
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

export const columns = [
  {
    title: '设备ID',
    key: 'id',
    align: 'left',
    width: 90,
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
    width: 180,
  },
  {
    title: '部署位置',
    key: 'location',
    align: 'left',
    width: 180,
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
  dict.setOption(OPS_DEVICE_GROUP_OPTION_KEY, options || []);
}
