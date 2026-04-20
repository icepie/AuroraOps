<template>
  <div class="device-page">
    <div class="device-layout" :class="{ 'device-layout--collapsed': groupCollapsed }">
      <div v-if="!groupCollapsed" class="device-layout__aside">
        <n-card :bordered="false" class="proCard group-panel" size="small">
          <template #header>
            <div class="group-header">
              <div class="group-header__heading">
                <div class="group-title">设备分组</div>
              </div>
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
      </div>
      <div class="device-layout__main">
        <n-card :bordered="false" class="proCard device-table-panel">
          <template #header>
            <div class="table-header">
              <div class="table-header__toggle-row">
                <n-button quaternary size="small" class="table-header__toggle" @click="toggleGroupPanel">
                  <template #icon>
                    <n-icon>
                      <component :is="groupCollapsed ? MenuUnfoldOutlined : MenuFoldOutlined" />
                    </n-icon>
                  </template>
                  {{ groupCollapsed ? '显示分组' : '隐藏分组' }}
                </n-button>
              </div>
              <div class="table-header__main">
                <div class="table-header__content">
                  <div class="table-header__title-row">
                    <div class="table-header__title">设备列表</div>
                    <n-tag size="small" round :bordered="false" class="table-header__tag">
                      {{ selectedGroupLabel }}
                    </n-tag>
                  </div>
                  <div class="table-header__subtitle">设备分组收起后，这里直接切换显示与隐藏。</div>
                </div>
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
      </div>
    </div>
    <Edit ref="editRef" @reload-table="handleReload" />
    <GroupModal ref="groupModalRef" @reload-groups="handleGroupReload" />
  </div>
</template>

<script lang="ts" setup>
  import { h, reactive, ref, computed, onMounted, onBeforeUnmount } from 'vue';
  import { useRouter } from 'vue-router';
  import { useDialog, useMessage, NButton, NIcon, NDropdown } from 'naive-ui';
  import { BasicTable } from '@/components/Table';
  import { BasicForm, useForm } from '@/components/Form/index';
  import { usePermission } from '@/hooks/web/usePermission';
  import { useDictStore } from '@/store/modules/dict';
  import { List, Delete, Status, CreateTerminal } from '@/api/opsDevice';
  import { Delete as DeleteGroup, List as GroupList } from '@/api/opsDeviceGroup';
  import {
    PlusOutlined,
    DeleteOutlined,
    SearchOutlined,
    EditOutlined,
    EllipsisOutlined,
    CodeOutlined,
    MenuFoldOutlined,
    MenuUnfoldOutlined,
  } from '@vicons/antd';
  import { columns, schemas, loadOptions, State, loadGroupOptions } from './model';
  import { adaTableScrollX } from '@/utils/hotgo';
  import Edit from './edit.vue';
  import GroupModal from './groupModal.vue';

  const dict = useDictStore();
  const dialog = useDialog();
  const message = useMessage();
  const router = useRouter();
  const { hasPermission } = usePermission();
  const actionRef = ref();
  const searchFormRef = ref<any>({});
  const editRef = ref();
  const groupModalRef = ref();
  const checkedIds = ref([]);
  const groupList = ref<any[]>([]);
  const activeGroupKey = ref<string>('all');
  const groupKeyword = ref('');
  const groupCollapsed = ref(false);
  let refreshTimer: ReturnType<typeof setInterval> | null = null;

  const actionColumn = reactive({
    width: 118,
    title: '操作',
    key: 'action',
    fixed: 'right',
    render(record: State) {
      const options = buildActionMenuOptions(record);

      return h('div', { class: 'device-action-cell' }, [
        h(
          NButton,
          {
            size: 'small',
            quaternary: true,
            type: record.online === true ? 'primary' : 'default',
            class: 'device-action-cell__terminal',
            onClick: handleTerminal.bind(null, record),
          },
          {
            icon: () =>
              h(
                NIcon,
                { size: 14 },
                {
                  default: () => h(CodeOutlined),
                },
              ),
            default: () => '终端',
          },
        ),
        options.length
          ? h(
              NDropdown,
              {
                trigger: 'click',
                options,
                onSelect: (key: string) => handleActionSelect(key, record),
              },
              {
                default: () =>
                  h(
                    NButton,
                    {
                      quaternary: true,
                      circle: true,
                      size: 'small',
                      class: 'device-action-cell__more',
                    },
                    {
                      icon: () =>
                        h(
                          NIcon,
                          { size: 16 },
                          {
                            default: () => h(EllipsisOutlined),
                          },
                        ),
                    },
                  ),
              },
            )
          : null,
      ]);
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

  function toggleGroupPanel() {
    groupCollapsed.value = !groupCollapsed.value;
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

  function buildActionMenuOptions(record: State) {
    const options: Array<{ label: string; key: string }> = [];

    if (hasPermission(['/opsDevice/edit'])) {
      options.push({ label: '编辑', key: 'edit' });
    }
    if (hasPermission(['/opsDevice/status'])) {
      if (record.status === 1) {
        options.push({ label: '禁用', key: 'disable' });
      } else if (record.status === 2) {
        options.push({ label: '启用', key: 'enable' });
      }
    }
    if (hasPermission(['/opsDevice/delete'])) {
      options.push({ label: '删除', key: 'delete' });
    }

    return options;
  }

  function handleActionSelect(key: string, record: Recordable) {
    switch (key) {
      case 'edit':
        handleEdit(record);
        break;
      case 'disable':
        handleStatus(record, 2);
        break;
      case 'enable':
        handleStatus(record, 1);
        break;
      case 'delete':
        handleDelete(record);
        break;
      default:
        break;
    }
  }

  async function handleTerminal(record: Recordable) {
    if (!record.online) {
      message.warning('设备已离线');
      return;
    }
    const res = await CreateTerminal({ deviceId: record.id });
    if (!res?.sessionId) {
      message.error('创建远程终端失败');
      return;
    }
    await router.push({
      name: 'ops_device_terminal_index',
      query: {
        sessionId: res.sessionId,
        deviceId: record.id,
        name: record.name || '',
      },
    });
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
    refreshTimer = setInterval(() => {
      reloadTable();
    }, 10000);
  });

  onBeforeUnmount(() => {
    if (refreshTimer) {
      clearInterval(refreshTimer);
      refreshTimer = null;
    }
  });
</script>

<style lang="less" scoped>
  .device-page {
    :deep(.n-card) {
      border-radius: 18px;
    }
  }

  .device-layout {
    display: grid;
    grid-template-columns: 300px minmax(0, 1fr);
    gap: 12px;
    align-items: start;
    transition: grid-template-columns 0.2s ease;
  }

  .device-layout--collapsed {
    grid-template-columns: minmax(0, 1fr);
  }

  .device-layout__aside,
  .device-layout__main {
    min-width: 0;
  }

  .group-panel {
    min-height: 100%;
    border: 1px solid rgba(148, 163, 184, 0.12);
    box-shadow: 0 8px 20px rgba(15, 23, 42, 0.03);
    background: #ffffff;
    overflow: hidden;
  }

  .group-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
  }

  .group-header__heading {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
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
    flex-direction: column;
    align-items: flex-start;
    gap: 10px;
  }

  .table-header__toggle-row {
    width: 100%;
  }

  .table-header__main {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
    width: 100%;
  }

  .table-header__toggle {
    flex: 0 0 auto;
    border-radius: 12px;
    padding: 0 12px;
    background: #f8fafc;
    border: 1px solid rgba(148, 163, 184, 0.16);
  }

  .table-header__content {
    min-width: 0;
  }

  .table-header__title-row {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    flex-wrap: wrap;
  }

  .table-header__title {
    color: #0f172a;
    font-size: 16px;
    font-weight: 700;
  }

  .table-header__tag {
    color: #475569;
    background: #f1f5f9;
  }

  .table-header__subtitle {
    margin-top: 4px;
    color: #64748b;
    font-size: 12px;
  }

  .device-action-cell {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 4px;
  }

  .device-action-cell__terminal {
    padding-left: 8px;
    padding-right: 8px;
    border-radius: 10px;
  }

  .device-action-cell__more {
    width: 28px;
    height: 28px;
    border-radius: 10px;
  }

  @media (max-width: 768px) {
    .device-layout,
    .device-layout--collapsed {
      grid-template-columns: 1fr;
    }

    .group-toolbar {
      align-items: flex-start;
      flex-direction: column;
    }

    .group-header {
      flex-wrap: wrap;
    }

    .soft-action-group {
      width: 100%;
      justify-content: flex-end;
    }

    .table-header__toggle {
      padding: 0 10px;
    }
  }
</style>
