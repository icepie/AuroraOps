import { h, ref } from 'vue';
import { cloneDeep } from 'lodash-es';
import { FormSchema } from '@/components/Form';
import { defRangeShortcuts } from '@/utils/dateUtil';
import { renderOptionTag } from '@/utils';
import { useDictStore } from '@/store/modules/dict';
import { Option as DeviceOption } from '@/api/opsDevice';

const dict = useDictStore();
export const OPS_DEVICE_OPTION_KEY = 'opsDeviceOptionRemote';

export const assetTypeOptions = [
  { label: '主板', value: 'motherboard', key: 'motherboard' },
  { label: 'CPU', value: 'cpu', key: 'cpu' },
  { label: '内存', value: 'memory', key: 'memory' },
  { label: '硬盘', value: 'disk', key: 'disk' },
  { label: '网卡', value: 'nic', key: 'nic' },
  { label: '电源', value: 'power', key: 'power' },
  { label: '风扇', value: 'fan', key: 'fan' },
  { label: '阵列卡', value: 'raid', key: 'raid' },
  { label: '其他', value: 'other', key: 'other' },
];

export class State {
  public id = 0;
  public deviceId = null;
  public deviceName = '';
  public assetType = 'motherboard';
  public assetName = '';
  public brand = '';
  public model = '';
  public serialNo = '';
  public specification = '';
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
  deviceId: {
    required: true,
    trigger: ['change'],
    message: '请选择所属设备',
  },
  assetType: {
    required: true,
    trigger: ['change'],
    message: '请选择资产类型',
  },
  assetName: {
    required: true,
    trigger: ['blur', 'input'],
    message: '请输入资产名称',
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
    label: '资产ID',
    componentProps: {
      placeholder: '请输入资产ID',
    },
  },
  {
    field: 'deviceId',
    component: 'NSelect',
    label: '所属设备',
    defaultValue: null,
    componentProps: {
      placeholder: '请选择所属设备',
      options: dict.getOption(OPS_DEVICE_OPTION_KEY),
      filterable: true,
      clearable: true,
    },
  },
  {
    field: 'assetType',
    component: 'NSelect',
    label: '资产类型',
    defaultValue: null,
    componentProps: {
      placeholder: '请选择资产类型',
      options: assetTypeOptions,
    },
  },
  {
    field: 'assetName',
    component: 'NInput',
    label: '资产名称',
    componentProps: {
      placeholder: '请输入资产名称',
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

function getAssetTypeLabel(value: string) {
  return assetTypeOptions.find((item) => item.value === value)?.label || value || '-';
}

export const columns = [
  {
    title: '资产ID',
    key: 'id',
    align: 'left',
    width: 90,
  },
  {
    title: '所属设备',
    key: 'deviceName',
    align: 'left',
    width: 160,
  },
  {
    title: '资产类型',
    key: 'assetType',
    align: 'left',
    width: 120,
    render(row: State) {
      return h('span', {}, getAssetTypeLabel(row.assetType));
    },
  },
  {
    title: '资产名称',
    key: 'assetName',
    align: 'left',
    width: 180,
  },
  {
    title: '品牌',
    key: 'brand',
    align: 'left',
    width: 140,
  },
  {
    title: '型号',
    key: 'model',
    align: 'left',
    width: 160,
  },
  {
    title: '序列号',
    key: 'serialNo',
    align: 'left',
    width: 180,
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

export async function loadOptions() {
  dict.loadOptions(['sys_normal_disable']);
  const options = await DeviceOption();
  dict.setOption(OPS_DEVICE_OPTION_KEY, options || []);
}
