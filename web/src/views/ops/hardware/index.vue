<template>
  <div class="hardware-page">
    <n-card :bordered="false" class="proCard hardware-card">
      <n-tabs v-model:value="activeTab" type="line" animated class="hardware-tabs">
        <n-tab-pane name="overview" tab="硬件信息">
          <div class="hardware-toolbar">
            <n-space :wrap="true" size="small" class="hardware-toolbar__filters">
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
            full-height
            ref="actionRef"
            :columns="columns"
            :request="loadDataTable"
            :row-key="(row) => row.deviceId"
            :actionColumn="actionColumn"
            :scroll-x="1840"
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
    display: flex;
    flex-direction: column;
    height: calc(100vh - 92px);
    min-height: 360px;
    overflow: hidden;
  }

  .hardware-card {
    display: flex;
    flex: 1 1 auto;
    flex-direction: column;
    min-height: 0;
    overflow: hidden;
  }

  .hardware-card :deep(.n-card-content) {
    display: flex;
    flex: 1 1 auto;
    flex-direction: column;
    min-height: 0;
  }

  .hardware-tabs {
    display: flex;
    flex: 1 1 auto;
    flex-direction: column;
    min-height: 0;
  }

  .hardware-tabs :deep(.n-tabs-nav) {
    flex: 0 0 auto;
    margin-bottom: 10px;
  }

  .hardware-tabs :deep(.n-tabs-pane-wrapper),
  .hardware-tabs :deep(.n-tab-pane) {
    display: flex;
    flex: 1 1 auto;
    flex-direction: column;
    min-height: 0;
  }

  .hardware-toolbar {
    flex: 0 0 auto;
    margin-bottom: 10px;
    padding: 8px 10px;
    border: 1px solid rgba(148, 163, 184, 0.16);
    border-radius: 8px;
    background: #f8fafc;
  }

  .hardware-tabs :deep(.basic-table) {
    flex: 1 1 auto;
    min-height: 280px;
  }

  .hardware-tabs :deep(.n-data-table-th),
  .hardware-tabs :deep(.n-data-table-td) {
    vertical-align: middle;
  }

  .hardware-tabs :deep(.n-data-table-td) {
    height: 56px;
  }

  .hardware-tabs :deep(.hardware-cell) {
    display: -webkit-box;
    max-height: 40px;
    overflow: hidden;
    line-height: 20px;
    white-space: normal;
    word-break: break-word;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
  }

  .hardware-tabs :deep(.hardware-cell--list) {
    display: flex;
    max-height: 46px;
    flex-wrap: wrap;
    align-content: flex-start;
    gap: 4px;
    line-height: 1;
  }

  .hardware-tabs :deep(.hardware-cell__item),
  .hardware-tabs :deep(.hardware-cell__more) {
    display: inline-flex;
    align-items: center;
    max-width: 100%;
    height: 20px;
    overflow: hidden;
    line-height: 20px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .hardware-toolbar__filters {
    align-items: center;
    row-gap: 8px;
  }

  .toolbar-select {
    width: 188px;
  }

  .toolbar-input {
    width: 320px;
  }

  .temperature-placeholder {
    min-height: 320px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  @media (min-width: 769px) and (max-width: 1280px) {
    .toolbar-input {
      width: 260px;
    }
  }

  @media (max-width: 768px) {
    .hardware-page {
      height: calc(100vh - 82px);
      min-height: 360px;
    }

    .hardware-toolbar {
      padding: 8px;
    }

    .hardware-toolbar__filters,
    .toolbar-select,
    .toolbar-input {
      width: 100%;
    }

    .hardware-toolbar :deep(.n-button) {
      flex: 1 1 120px;
    }

    .temperature-placeholder {
      min-height: 240px;
    }
  }
</style>
