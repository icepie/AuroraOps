<template>
  <div class="device-page">
    <n-grid cols="1 s:1 m:1 l:4 xl:4 2xl:4" responsive="screen" :x-gap="12">
      <n-gi span="1">
        <n-card :bordered="false" class="proCard group-panel" size="small">
          <template #header>
            <div class="group-header">
              <div class="group-title">设备分组</div>
              <n-space size="small" class="soft-action-group">
                <n-tooltip v-if="hasPermission(['/opsDeviceGroup/edit'])" trigger="hover">
                  <template #trigger>
                    <n-button
                      size="small"
                      type="primary"
                      quaternary
                      circle
                      @click="openGroupModal()"
                    >
                      <template #icon>
                        <n-icon><PlusOutlined /></n-icon>
                      </template>
                    </n-button>
                  </template>
                  新增分组
                </n-tooltip>
                <n-tooltip v-if="hasPermission(['/opsDeviceGroup/edit'])" trigger="hover">
                  <template #trigger>
                    <n-button
                      size="small"
                      quaternary
                      circle
                      @click="openGroupModal(selectedGroupRecord)"
                      :disabled="!selectedGroupRecord"
                    >
                      <template #icon>
                        <n-icon><EditOutlined /></n-icon>
                      </template>
                    </n-button>
                  </template>
                  编辑分组
                </n-tooltip>
                <n-tooltip v-if="hasPermission(['/opsDeviceGroup/delete'])" trigger="hover">
                  <template #trigger>
                    <n-button
                      size="small"
                      type="error"
                      quaternary
                      circle
                      @click="handleGroupDelete"
                      :disabled="!selectedGroupRecord"
                    >
                      <template #icon>
                        <n-icon><DeleteOutlined /></n-icon>
                      </template>
                    </n-button>
                  </template>
                  删除分组
                </n-tooltip>
              </n-space>
            </div>
          </template>
          <div class="group-panel__body">
            <div class="group-toolbar">
              <n-space size="small" wrap>
                <n-tag round :bordered="false">分组 {{ groupList.length }}</n-tag>
                <n-tag round type="info" :bordered="false">已归组 {{ groupedDeviceCount }}</n-tag>
              </n-space>
              <n-alert :show-icon="false" type="default" class="group-current-alert">
                当前：{{ selectedGroupLabel }}
              </n-alert>
            </div>
            <n-input
              v-model:value="groupKeyword"
              clearable
              placeholder="搜索分组名称"
              class="group-search"
            >
              <template #suffix>
                <n-icon size="16">
                  <SearchOutlined />
                </n-icon>
              </template>
            </n-input>
            <n-divider class="group-divider">分组列表</n-divider>
            <div class="group-menu-shell">
              <n-empty
                v-if="filteredGroupCount === 0"
                size="small"
                description="没有匹配的分组"
                class="group-empty"
              />
              <n-scrollbar v-else style="max-height: 560px">
                <n-menu
                  :value="activeGroupKey"
                  :options="groupOptions"
                  @update:value="handleGroupChange"
                />
              </n-scrollbar>
            </div>
          </div>
        </n-card>
      </n-gi>
      <n-gi span="3">
        <n-card :bordered="false" class="proCard device-table-panel">
          <template #header>
            <div class="table-header">
              <div>
                <div class="table-header__title">设备列表</div>
                <div class="table-header__subtitle">当前筛选：{{ selectedGroupLabel }}</div>
              </div>
            </div>
          </template>
          <BasicForm
            ref="searchFormRef"
            @register="register"
            @submit="reloadTable"
            @reset="reloadTable"
            @keyup.enter="reloadTable"
          />
          <BasicTable
            ref="actionRef"
            openChecked
            :columns="columns"
            :request="loadDataTable"
            :row-key="(row) => row.id"
            :actionColumn="actionColumn"
            :scroll-x="scrollX"
            :resizeHeightOffset="-10000"
            :checked-row-keys="checkedIds"
            @update:checked-row-keys="handleOnCheckedRow"
          >
            <template #tableTitle>
              <n-button
                v-if="hasPermission(['/opsDevice/edit'])"
                type="primary"
                secondary
                class="min-left-space"
                @click="addTable"
              >
                <template #icon>
                  <n-icon><PlusOutlined /></n-icon>
                </template>
                新增设备
              </n-button>
              <n-button
                v-if="hasPermission(['/opsDevice/delete'])"
                type="error"
                secondary
                class="min-left-space"
                @click="handleBatchDelete"
              >
                <template #icon>
                  <n-icon><DeleteOutlined /></n-icon>
                </template>
                批量删除
              </n-button>
            </template>
          </BasicTable>
        </n-card>
      </n-gi>
    </n-grid>
    <Edit ref="editRef" @reload-table="handleReload" />
    <GroupModal ref="groupModalRef" @reload-groups="handleGroupReload" />
  </div>
</template>

<script lang="ts" setup>
  import { h, reactive, ref, computed, onMounted } from 'vue';
  import { useDialog, useMessage } from 'naive-ui';
  import { BasicTable, TableAction } from '@/components/Table';
  import { BasicForm, useForm } from '@/components/Form/index';
  import { usePermission } from '@/hooks/web/usePermission';
  import { useDictStore } from '@/store/modules/dict';
  import { List, Delete, Status } from '@/api/opsDevice';
  import { Delete as DeleteGroup, List as GroupList } from '@/api/opsDeviceGroup';
  import { PlusOutlined, DeleteOutlined, SearchOutlined, EditOutlined } from '@vicons/antd';
  import { columns, schemas, loadOptions, State, loadGroupOptions } from './model';
  import { adaTableScrollX } from '@/utils/hotgo';
  import Edit from './edit.vue';
  import GroupModal from './groupModal.vue';

  const dict = useDictStore();
  const dialog = useDialog();
  const message = useMessage();
  const { hasPermission } = usePermission();
  const actionRef = ref();
  const searchFormRef = ref<any>({});
  const editRef = ref();
  const groupModalRef = ref();
  const checkedIds = ref([]);
  const groupList = ref<any[]>([]);
  const activeGroupKey = ref<string>('all');
  const groupKeyword = ref('');

  const actionColumn = reactive({
    width: 216,
    title: '操作',
    key: 'action',
    fixed: 'right',
    render(record: State) {
      return h(TableAction as any, {
        style: 'button',
        actions: [
          {
            label: '编辑',
            onClick: handleEdit.bind(null, record),
            auth: ['/opsDevice/edit'],
          },
          {
            label: '禁用',
            onClick: handleStatus.bind(null, record, 2),
            ifShow: () => record.status === 1,
            auth: ['/opsDevice/status'],
          },
          {
            label: '启用',
            onClick: handleStatus.bind(null, record, 1),
            ifShow: () => record.status === 2,
            auth: ['/opsDevice/status'],
          },
          {
            label: '删除',
            onClick: handleDelete.bind(null, record),
            auth: ['/opsDevice/delete'],
          },
        ],
      });
    },
  });

  const scrollX = computed(() => adaTableScrollX(columns, actionColumn.width));

  const [register] = useForm({
    gridProps: { cols: '1 s:1 m:2 l:3 xl:4 2xl:4' },
    labelWidth: 80,
    schemas,
  });

  const groupedDeviceCount = computed(() => {
    return groupList.value.reduce((total, item) => total + Number(item.deviceCount || 0), 0);
  });

  const selectedGroupLabel = computed(() => {
    if (activeGroupKey.value === 'all') {
      return '全部设备';
    }
    if (activeGroupKey.value === 'ungrouped') {
      return '未分组';
    }
    return selectedGroupRecord.value?.name || '设备分组';
  });

  const groupOptions = computed(() => {
    const keyword = groupKeyword.value.trim().toLowerCase();
    const options = [
      { label: '全部设备', key: 'all' },
      { label: '未分组', key: 'ungrouped' },
    ];

    const visibleGroups = keyword
      ? groupList.value.filter((item) =>
          String(item.name || '')
            .toLowerCase()
            .includes(keyword)
        )
      : groupList.value;

    return options.concat(
      visibleGroups.map((item) => ({
        label: `${item.name}${typeof item.deviceCount === 'number' ? ` (${item.deviceCount})` : ''}`,
        key: `group-${item.id}`,
      }))
    );
  });

  const filteredGroupCount = computed(() => {
    return groupKeyword.value.trim()
      ? Math.max(groupOptions.value.length - 2, 0)
      : groupList.value.length;
  });

  const selectedGroupRecord = computed(() => {
    if (!activeGroupKey.value.startsWith('group-')) {
      return null;
    }
    const id = Number(activeGroupKey.value.replace('group-', ''));
    return groupList.value.find((item) => item.id === id) || null;
  });

  const loadDataTable = async (res) => {
    const params: Record<string, any> = { ...searchFormRef.value?.formModel, ...res };
    if (activeGroupKey.value === 'ungrouped') {
      params.groupScope = 'ungrouped';
    } else if (activeGroupKey.value.startsWith('group-')) {
      params.groupId = Number(activeGroupKey.value.replace('group-', ''));
    }
    return await List(params);
  };

  function handleOnCheckedRow(rowKeys) {
    checkedIds.value = rowKeys;
  }

  function reloadTable() {
    actionRef.value?.reload();
  }

  async function loadGroups() {
    const res = await GroupList();
    groupList.value = res?.list || [];

    if (
      activeGroupKey.value.startsWith('group-') &&
      !groupList.value.some((item) => `group-${item.id}` === activeGroupKey.value)
    ) {
      activeGroupKey.value = 'all';
    }
  }

  function handleGroupChange(key: string) {
    activeGroupKey.value = key;
    reloadTable();
  }

  function openGroupModal(record: Recordable | null = null) {
    groupModalRef.value?.openModal(record);
  }

  function handleGroupReload() {
    loadGroups();
    loadGroupOptions();
    reloadTable();
  }

  function handleReload() {
    reloadTable();
    loadGroups();
  }

  function addTable() {
    editRef.value.openModal(null);
  }

  function handleEdit(record: Recordable) {
    editRef.value.openModal(record);
  }

  function handleDelete(record: Recordable) {
    dialog.warning({
      title: '警告',
      content: '你确定要删除该设备？',
      positiveText: '确定',
      negativeText: '取消',
      onPositiveClick: () => {
        Delete(record).then(() => {
          message.success('删除成功');
          handleReload();
        });
      },
    });
  }

  function handleBatchDelete() {
    if (checkedIds.value.length < 1) {
      message.error('请至少选择一项要删除的数据');
      return;
    }

    dialog.warning({
      title: '警告',
      content: '你确定要批量删除设备？',
      positiveText: '确定',
      negativeText: '取消',
      onPositiveClick: () => {
        Delete({ id: checkedIds.value }).then(() => {
          checkedIds.value = [];
          message.success('删除成功');
          handleReload();
        });
      },
    });
  }

  function handleStatus(record: Recordable, status: number) {
    Status({ id: record.id, status }).then(() => {
      message.success('设为' + dict.getLabel('sys_normal_disable', status) + '成功');
      handleReload();
    });
  }

  function handleGroupDelete() {
    if (!selectedGroupRecord.value) {
      return;
    }

    dialog.warning({
      title: '警告',
      content: `你确定要删除分组“${selectedGroupRecord.value.name}”吗？`,
      positiveText: '确定',
      negativeText: '取消',
      onPositiveClick: () => {
        DeleteGroup({ id: selectedGroupRecord.value.id }).then(() => {
          message.success('删除成功');
          activeGroupKey.value = 'all';
          handleGroupReload();
        });
      },
    });
  }

  onMounted(async () => {
    loadOptions();
    await loadGroupOptions();
    await loadGroups();
  });
</script>

<style lang="less" scoped>
  .device-page {
    :deep(.n-card) {
      border-radius: 18px;
    }
  }

  .group-panel {
    min-height: 100%;
    border: 1px solid rgba(148, 163, 184, 0.12);
    box-shadow: 0 8px 20px rgba(15, 23, 42, 0.03);
    background: #ffffff;
  }

  .group-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
  }

  .group-title {
    color: #0f172a;
    font-size: 16px;
    font-weight: 700;
  }

  .group-panel__body {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .group-header :deep(.n-button-group) {
    flex-wrap: wrap;
  }

  .soft-action-group :deep(.n-button) {
    border-radius: 10px;
  }

  .soft-action-group :deep(.n-button__content) {
    font-weight: 600;
  }

  .group-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: wrap;
  }

  .group-current-alert {
    min-width: 168px;
    border-radius: 12px;
    background: #f8fafc;
    border: 1px solid rgba(148, 163, 184, 0.14);

    :deep(.n-alert-body) {
      padding: 8px 12px;
    }
  }

  .group-search {
    :deep(.n-input) {
      border-radius: 12px;
      background: #ffffff;
    }
  }

  .group-divider {
    margin: 0;
    color: #64748b;
    font-size: 12px;
  }

  .group-menu-shell {
    padding: 8px;
    border-radius: 16px;
    background: #fafafa;
    border: 1px solid rgba(148, 163, 184, 0.12);
  }

  .group-empty {
    padding: 20px 0 10px;
  }

  .group-menu-shell :deep(.n-menu) {
    background: transparent;
  }

  .group-menu-shell :deep(.n-menu-item),
  .group-menu-shell :deep(.n-menu-item-content),
  .group-menu-shell :deep(.n-menu-item-content-header) {
    border-radius: 12px;
  }

  .group-menu-shell :deep(.n-menu-item-content) {
    margin: 4px 0;
    min-height: 44px;
    transition:
      background-color 0.18s ease,
      transform 0.18s ease;
  }

  .group-menu-shell :deep(.n-menu-item-content:hover) {
    transform: translateX(2px);
  }

  .device-table-panel {
    border: 1px solid rgba(148, 163, 184, 0.1);
    box-shadow: 0 8px 20px rgba(15, 23, 42, 0.03);
    background: #ffffff;
  }

  .table-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: wrap;
  }

  .table-header__title {
    color: #0f172a;
    font-size: 16px;
    font-weight: 700;
  }

  .table-header__subtitle {
    margin-top: 4px;
    color: #64748b;
    font-size: 13px;
  }

  .table-header__meta {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 8px;
  }

  .table-header__meta--compact {
    align-self: center;
  }

  @media (max-width: 768px) {
    .group-toolbar {
      align-items: flex-start;
      flex-direction: column;
    }
  }
</style>
