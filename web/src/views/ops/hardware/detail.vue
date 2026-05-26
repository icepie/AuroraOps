<template>
  <n-modal
    v-model:show="showModal"
    preset="card"
    :bordered="false"
    :segmented="{ content: true }"
    :mask-closable="true"
    style="width: min(1080px, 92vw)"
    :title="modalTitle"
  >
    <n-spin :show="loading" description="加载硬件明细中...">
      <n-data-table
        :columns="columns"
        :data="tableData"
        :pagination="false"
        :row-key="(row) => row.id"
        size="small"
        max-height="560"
        :scroll-x="1500"
      />
    </n-spin>
  </n-modal>
</template>

<script lang="ts" setup>
  import { h, ref } from 'vue';
  import { NTag } from 'naive-ui';
  import { List as AssetList } from '@/api/opsAsset';

  const showModal = ref(false);
  const loading = ref(false);
  const modalTitle = ref('硬件明细');
  const tableData = ref<any[]>([]);

  const assetTypeLabelMap = {
    motherboard: '主板',
    bios: 'BIOS',
    cpu: 'CPU',
    memory: '内存',
    disk: '磁盘',
    storage: '磁盘',
    physical_disk: '磁盘',
    physicaldisk: '磁盘',
    drive: '磁盘',
    nic: '网卡',
    network: '网卡',
    network_interface: '网卡',
    gpu: '显卡',
    video: '显卡',
    graphics: '显卡',
    graphics_card: '显卡',
    power: '电源',
    fan: '风扇',
    raid: '阵列卡',
    other: '其他',
  };

  const columns = [
    {
      title: '硬件类型',
      key: 'assetType',
      width: 110,
      render(row) {
        return assetTypeLabelMap[row.assetType] || row.assetType || '-';
      },
    },
    {
      title: '硬件名称',
      key: 'assetName',
      minWidth: 180,
      ellipsis: { tooltip: true },
    },
    {
      title: '品牌',
      key: 'brand',
      width: 130,
      ellipsis: { tooltip: true },
    },
    {
      title: '型号',
      key: 'model',
      minWidth: 160,
      ellipsis: { tooltip: true },
    },
    {
      title: '唯一键',
      key: 'uniqueKey',
      minWidth: 180,
      ellipsis: { tooltip: true },
    },
    {
      title: '序列号',
      key: 'serialNo',
      minWidth: 170,
      ellipsis: { tooltip: true },
    },
    {
      title: '规格参数',
      key: 'specification',
      minWidth: 220,
      ellipsis: { tooltip: true },
    },
    {
      title: '来源',
      key: 'source',
      width: 90,
      render(row) {
        return isAutoSource(row.source) ? '自动采集' : '手动';
      },
    },
    {
      title: '状态',
      key: 'status',
      width: 90,
      render(row) {
        return h(
          NTag,
          {
            bordered: false,
            type: row.status === 1 ? 'success' : 'default',
          },
          {
            default: () => (row.status === 1 ? '正常' : '停用'),
          }
        );
      },
    },
    {
      title: '最近观测',
      key: 'lastSeenAt',
      width: 170,
      render(row) {
        return row.lastSeenAt || row.createdAt || '-';
      },
    },
  ];

  async function open(device: { deviceId: number; deviceName?: string }) {
    modalTitle.value = `${device.deviceName || '设备'} 硬件明细`;
    showModal.value = true;
    loading.value = true;
    try {
      const res = await AssetList({
        deviceId: device.deviceId,
        page: 1,
        perPage: 200,
      });
      tableData.value = res?.list || [];
    } finally {
      loading.value = false;
    }
  }

  function isAutoSource(source?: string) {
    return ['agent', 'auroraops-agent', 'fastfetch-sys'].includes(source || '');
  }

  defineExpose({
    open,
  });
</script>
