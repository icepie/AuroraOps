import { cloneDeep } from 'lodash-es';
import type { FormRules } from 'naive-ui/es/form/src/interface';
import { useDictStore } from '@/store/modules/dict';
import { Option as DeviceOption } from '@/api/opsDevice';

const dict = useDictStore();

export const OPS_DEVICE_OPTION_KEY = 'opsDeviceOptionRemote';

export const assetTypeOptions = [
  { label: '主板', value: 'motherboard', key: 'motherboard' },
  { label: 'BIOS', value: 'bios', key: 'bios' },
  { label: 'CPU', value: 'cpu', key: 'cpu' },
  { label: '内存', value: 'memory', key: 'memory' },
  { label: '磁盘', value: 'disk', key: 'disk' },
  { label: '网卡', value: 'network', key: 'network' },
  { label: '显卡', value: 'gpu', key: 'gpu' },
  { label: '电源', value: 'power', key: 'power' },
  { label: '风扇', value: 'fan', key: 'fan' },
  { label: '阵列卡', value: 'raid', key: 'raid' },
  { label: '其他', value: 'other', key: 'other' },
];

export class State {
  public id = 0;
  public deviceId = null;
  public assetType = 'other';
  public uniqueKey = '';
  public assetName = '';
  public brand = '';
  public model = '';
  public serialNo = '';
  public specification = '';
  public source = 'manual';
  public syncHash = '';
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

export const rules: FormRules = {
  deviceId: {
    required: true,
    trigger: ['change', 'blur'],
    type: 'number',
    message: '请选择所属设备',
  },
  assetType: {
    required: true,
    trigger: ['change', 'blur'],
    message: '请选择硬件类型',
  },
  assetName: {
    required: true,
    trigger: ['input', 'blur'],
    message: '请输入硬件名称',
  },
  sort: {
    required: true,
    trigger: ['change', 'blur'],
    type: 'number',
    message: '请输入排序',
  },
};

export function loadOptions() {
  dict.loadOptions(['sys_normal_disable']);
}

export async function loadDeviceOptions() {
  const options = await DeviceOption();
  dict.setOption(OPS_DEVICE_OPTION_KEY, options || []);
}

export async function loadOptionsAndDict() {
  loadOptions();
  await loadDeviceOptions();
}
