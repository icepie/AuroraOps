<template>
  <div class="hardware-page">
    <n-card :bordered="false" class="proCard hardware-card">
      <n-tabs v-model:value="activeTab" type="line" animated class="hardware-tabs">
        <n-tab-pane name="overview" tab="硬件信息">
          <div class="hardware-toolbar">
            <n-space :wrap="false" size="small" class="hardware-toolbar__filters">
              <n-select
                v-model:value="filters.groupId"
                :options="groupOptions"
                class="toolbar-select"
                placeholder="所有分组"
                clearable
              />
              <n-input
                v-model:value="filters.keyword"
                clearable
                class="toolbar-input"
                placeholder="请输入机器名查询"
                @keyup.enter="reloadTable"
              />
              <n-checkbox v-model:checked="filters.onlyChanged">仅显示变动项</n-checkbox>
              <n-button type="primary" @click="reloadTable">查询</n-button>
              <n-button @click="handleExport">导出硬件列表</n-button>
            </n-space>
          </div>

          <BasicTable
            ref="actionRef"
            :columns="columns"
            :request="loadDataTable"
            :row-key="(row) => row.deviceId"
            :actionColumn="actionColumn"
            :scroll-x="1440"
            :resizeHeightOffset="-10000"
            :showTopRight="false"
          />
        </n-tab-pane>

        <n-tab-pane name="temperature" tab="温度监控">
          <n-empty description="温度监控将在第二阶段接入采集数据" class="temperature-placeholder" />
        </n-tab-pane>
      </n-tabs>
    </n-card>

    <Detail ref="detailRef" />
  </div>
</template>

<script lang="ts" setup>
  import { h, reactive, ref, onMounted } from 'vue';
  import { useMessage } from 'naive-ui';
  import { BasicTable, TableAction } from '@/components/Table';
  import { List as GroupList } from '@/api/opsDeviceGroup';
  import { Overview, Export } from '@/api/opsHardware';
  import { columns } from './overview-model';
  import Detail from './detail.vue';

  const message = useMessage();
  const activeTab = ref('overview');
  const actionRef = ref();
  const detailRef = ref();
  const groupOptions = ref<{ label: string; value: number | null }[]>([
    { label: '所有分组', value: null },
  ]);

  const filters = reactive({
    groupId: null as number | null,
    keyword: '',
    onlyChanged: false,
  });

  const actionColumn = reactive({
    width: 120,
    title: '操作',
    key: 'action',
    fixed: 'right',
    render(record) {
      return h(TableAction as any, {
        style: 'button',
        actions: [
          {
            label: '查看明细',
            onClick: () => handleViewDetail(record),
          },
        ],
      });
    },
  });

  const loadDataTable = async (params) => {
    return await Overview({
      ...params,
      groupId: filters.groupId || undefined,
      keyword: filters.keyword,
      onlyChanged: filters.onlyChanged,
    });
  };

  function reloadTable() {
    actionRef.value?.reload();
  }

  function handleExport() {
    message.loading('正在导出硬件列表...', { duration: 1200 });
    Export({
      groupId: filters.groupId || undefined,
      keyword: filters.keyword,
      onlyChanged: filters.onlyChanged,
    });
  }

  function handleViewDetail(record) {
    detailRef.value?.open(record);
  }

  async function loadGroups() {
    const list = await GroupList();
    groupOptions.value = [
      { label: '所有分组', value: null },
      ...(list || []).map((item) => ({
        label: item.name,
        value: item.id,
      })),
    ];
  }

  onMounted(async () => {
    await loadGroups();
  });
</script>

<style scoped>
  .hardware-page {
    min-height: calc(100vh - 180px);
  }

  .hardware-card {
    overflow: hidden;
  }

  .hardware-tabs :deep(.n-tabs-nav) {
    margin-bottom: 16px;
  }

  .hardware-toolbar {
    margin-bottom: 16px;
  }

  .hardware-toolbar__filters {
    align-items: center;
  }

  .toolbar-select {
    width: 188px;
  }

  .toolbar-input {
    width: 320px;
  }

  .temperature-placeholder {
    min-height: 360px;
    display: flex;
    align-items: center;
    justify-content: center;
  }
</style>
