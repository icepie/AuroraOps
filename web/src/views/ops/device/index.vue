<template>
  <div ref="devicePageRef" class="device-page" :style="{ '--device-page-height': devicePageHeight }">
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
                <n-button
                  quaternary
                  size="small"
                  class="table-header__toggle"
                  @click="toggleGroupPanel"
                >
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
           <BasicTable full-height
            ref="actionRef"
            openChecked
            :columns="columns"
            :request="loadDataTable"
            :row-key="(row) => row.id"
            :actionColumn="tableActionColumn"
            :scroll-x="scrollX"
            :checked-row-keys="checkedIds"
            :expanded-row-keys="expandedRowKeys"
            @update:checked-row-keys="handleOnCheckedRow"
            @update:expanded-row-keys="handleExpandedRowKeys"
          >
            <template #tableTitle>
              <div class="device-table-actions">
                <n-button
                  v-if="hasPermission(['/opsDevice/edit'])"
                  type="primary"
                  secondary
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
                  @click="handleBatchDelete"
                >
                  <template #icon>
                    <n-icon><DeleteOutlined /></n-icon>
                  </template>
                  批量删除
                </n-button>
              </div>
            </template>
            <template #toolbar>
              <n-button secondary size="small" @click="handleExpandAll">展开全部</n-button>
              <n-button secondary size="small" @click="handleCollapseAll">折叠全部</n-button>
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
  import { h, reactive, ref, computed, onMounted, onBeforeUnmount, nextTick } from 'vue';
  import type { Component } from 'vue';
  import { useRouter } from 'vue-router';
  import { useDialog, useMessage, NButton, NIcon, NDropdown, NTooltip } from 'naive-ui';
  import { BasicTable } from '@/components/Table';
  import { BasicForm, useForm } from '@/components/Form/index';
  import { usePermission } from '@/hooks/web/usePermission';
  import { useDictStore } from '@/store/modules/dict';
  import { List, Delete, Status, Wake, CreateTerminal, CreateDesktop } from '@/api/opsDevice';
  import { Delete as DeleteGroup, List as GroupList } from '@/api/opsDeviceGroup';
  import { SocketEnum } from '@/enums/socketEnum';
  import { addOnMessage, removeOnMessage, sendMsg, WebSocketMessage } from '@/utils/websocket';
  import {
    PlusOutlined,
    DeleteOutlined,
    SearchOutlined,
    EditOutlined,
    EllipsisOutlined,
    CodeOutlined,
    DesktopOutlined,
    ThunderboltOutlined,
    MenuFoldOutlined,
    MenuUnfoldOutlined,
  } from '@vicons/antd';
  import {
    columns,
    schemas,
    loadOptions,
    State,
    loadGroupOptions,
    DEVICE_MONITOR_EVENT,
    DEVICE_MONITOR_TAG,
  } from './model';
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
  const devicePageRef = ref<HTMLElement | null>(null);
  const devicePageHeight = ref('calc(100vh - 92px)');
  const checkedIds = ref([]);
  const groupList = ref<any[]>([]);
  const activeGroupKey = ref<string>('all');
  const groupKeyword = ref('');
  const groupCollapsed = ref(false);
  const expandedRowKeys = ref<Array<number | string>>([]);
  const isCompactDeviceTable = ref(false);
  let deviceResizeObserver: ResizeObserver | null = null;

  function updateDevicePageHeight() {
    const el = devicePageRef.value;
    if (!el?.getBoundingClientRect) return;
    const top = el.getBoundingClientRect().top;
    const viewportHeight = window.document.documentElement.clientHeight || window.innerHeight;
    const bottomGap = 18;
    devicePageHeight.value = `${Math.max(360, Math.floor(viewportHeight - top - bottomGap))}px`;
    isCompactDeviceTable.value = el.clientWidth > 0 && el.clientWidth <= 1180;
  }

  function renderActionButton(
    label: string,
    icon: Component,
    onClick: () => void,
    type: 'default' | 'primary' | 'warning' = 'default'
  ) {
    return h(
      NTooltip,
      { trigger: 'hover' },
      {
        trigger: () =>
          h(
            NButton,
            {
              size: 'small',
              quaternary: true,
              circle: true,
              type,
              class: 'device-action-cell__button',
              onClick,
            },
            {
              icon: () =>
                h(
                  NIcon,
                  { size: 15 },
                  {
                    default: () => h(icon),
                  }
                ),
            }
          ),
        default: () => label,
      }
    );
  }

  function renderWakeButton(record: State) {
    return renderActionButton(
      record.macAddress ? '发送网络唤醒' : '缺少MAC地址',
      ThunderboltOutlined,
      handleWake.bind(null, record),
      record.macAddress ? 'warning' : 'default'
    );
  }

  const actionColumn = reactive({
    width: 156,
    title: '操作',
    key: 'action',
    fixed: 'right',
    render(record: State) {
      const options = buildActionMenuOptions(record);

      return h('div', { class: 'device-action-cell' }, [
        renderActionButton(
          '远程终端',
          CodeOutlined,
          handleTerminal.bind(null, record),
          record.online === true ? 'primary' : 'default'
        ),
        renderActionButton(
          '远程桌面',
          DesktopOutlined,
          handleDesktop.bind(null, record),
          record.online === true ? 'primary' : 'default'
        ),
        hasPermission(['/opsDevice/wake']) ? renderWakeButton(record) : null,
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
                      class: 'device-action-cell__button',
                    },
                    {
                      icon: () =>
                        h(
                          NIcon,
                          { size: 16 },
                          {
                            default: () => h(EllipsisOutlined),
                          }
                        ),
                    }
                  ),
              }
            )
          : null,
      ]);
    },
  });

  const scrollX = computed(() => adaTableScrollX(columns, actionColumn.width));
  const tableActionColumn = computed(() => ({
    ...actionColumn,
    fixed: isCompactDeviceTable.value ? undefined : 'right',
  }));

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

  function handleExpandedRowKeys(rowKeys) {
    expandedRowKeys.value = rowKeys;
  }

  function handleExpandAll() {
    const rows = actionRef.value?.getDataSource?.();
    if (!Array.isArray(rows)) return;
    expandedRowKeys.value = rows.map((item) => item.id).filter((id) => id !== undefined && id !== null);
  }

  function handleCollapseAll() {
    expandedRowKeys.value = [];
  }

  function reloadTable() {
    actionRef.value?.reload();
  }

  function applyMonitorUpdate(payload: any) {
    const deviceId = Number(payload?.deviceId || 0);
    if (!deviceId) return;

    const rows = actionRef.value?.getDataSource?.();
    if (!Array.isArray(rows)) return;

    const index = rows.findIndex((item) => Number(item.id) === deviceId);
    if (index < 0) return;

    const nextRows = rows.slice();
    nextRows[index] = {
      ...nextRows[index],
      monitor: payload.monitor || null,
      monitorReportedAt: payload.monitorReportedAt || '',
      online: true,
    };
    actionRef.value?.setTableData?.(nextRows);
  }

  function joinMonitorChannel() {
    sendMsg('join', { id: DEVICE_MONITOR_TAG }, false);
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
    nextTick(updateDevicePageHeight);
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

  async function handleDesktop(record: Recordable) {
    if (!record.online) {
      message.warning('设备已离线');
      return;
    }
    const res = await CreateDesktop({ deviceId: record.id });
    if (!res?.sessionId) {
      message.error('创建远程桌面失败');
      return;
    }
    await router.push({
      name: 'ops_device_desktop_index',
      query: {
        sessionId: res.sessionId,
        deviceId: record.id,
        name: record.name || '',
      },
    });
  }

  function handleWake(record: Recordable) {
    if (!record.macAddress) {
      message.warning('该设备没有MAC地址，无法发送网络唤醒');
      return;
    }
    dialog.warning({
      title: '网络唤醒',
      content: `确认向“${record.name || record.hostname || record.id}”发送 Wake-on-LAN 魔术包？`,
      positiveText: '发送',
      negativeText: '取消',
      onPositiveClick: () => {
        return Wake({ id: record.id }).then((res) => {
          const packets = Number(res?.packets || 0);
          const targets = Array.isArray(res?.targets) ? res.targets.join('、') : '';
          message.success(`已发送${packets ? ` ${packets} 个` : ''}唤醒包${targets ? `：${targets}` : ''}`);
        });
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
    nextTick(() => {
      updateDevicePageHeight();
      if (devicePageRef.value) {
        deviceResizeObserver = new ResizeObserver(updateDevicePageHeight);
        deviceResizeObserver.observe(devicePageRef.value);
      }
    });
    window.addEventListener('resize', updateDevicePageHeight);
    loadOptions();
    addOnMessage(SocketEnum.EventConnected, joinMonitorChannel);
    addOnMessage(DEVICE_MONITOR_EVENT, (message: WebSocketMessage) => {
      if (message.code === SocketEnum.CodeErr) {
        return;
      }
      applyMonitorUpdate(message.data);
    });
    joinMonitorChannel();
    await loadGroupOptions();
    await loadGroups();
  });

  onBeforeUnmount(() => {
    window.removeEventListener('resize', updateDevicePageHeight);
    deviceResizeObserver?.disconnect();
    deviceResizeObserver = null;
    sendMsg('quit', { id: DEVICE_MONITOR_TAG }, false);
    removeOnMessage(SocketEnum.EventConnected);
    removeOnMessage(DEVICE_MONITOR_EVENT);
  });
</script>

<style lang="less" scoped>
  .device-page {
    width: 100%;
    height: var(--device-page-height, calc(100vh - 92px));
    min-height: 360px;
    overflow: hidden;
    container-type: inline-size;
    min-width: 0;

    :deep(.n-card) {
      border-radius: 4px;
    }
  }

  .device-layout {
    display: grid;
    grid-template-columns: 272px minmax(0, 1fr);
    width: 100%;
    min-width: 0;
    gap: 8px;
    align-items: stretch;
    height: 100%;
    min-height: 0;
    transition: grid-template-columns 0.2s ease;
  }

  .device-layout--collapsed {
    grid-template-columns: minmax(0, 1fr);
  }

  .device-layout__aside,
  .device-layout__main {
    min-width: 0;
    min-height: 0;
  }

  .device-layout__main {
    display: flex;
    flex: 1 1 auto;
    flex-direction: column;
    width: 100%;
    max-width: 100%;
  }

  .group-panel {
    height: 100%;
    border: 1px solid var(--n-border-color);
    box-shadow: none;
    background: var(--n-color);
    overflow: hidden;
  }

  .group-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 8px;
  }

  .group-header__heading {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .group-title {
    color: var(--n-title-text-color);
    font-size: 14px;
    font-weight: 600;
  }

  .group-panel__body {
    display: flex;
    flex-direction: column;
    gap: 9px;
  }

  .group-header :deep(.n-button-group) {
    flex-wrap: wrap;
  }

  .soft-action-group :deep(.n-button) {
    border-radius: 4px;
  }

  .soft-action-group :deep(.n-button__content) {
    font-weight: 600;
  }

  .group-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    flex-wrap: wrap;
  }

  .group-current-alert {
    min-width: 148px;
    border-radius: 4px;
    background: var(--n-merged-color);
    border: 1px solid var(--n-border-color);

    :deep(.n-alert-body) {
      padding: 5px 8px;
    }
  }

  .group-search {
    :deep(.n-input) {
      border-radius: 4px;
    }
  }

  .group-divider {
    margin: 0;
    color: var(--n-text-color-3);
    font-size: 12px;
  }

  .group-menu-shell {
    padding: 5px;
    border-radius: 4px;
    background: var(--n-merged-color);
    border: 1px solid var(--n-border-color);
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
    border-radius: 4px;
  }

  .group-menu-shell :deep(.n-menu-item-content) {
    margin: 2px 0;
    min-height: 34px;
    transition: background-color 0.12s ease;
  }

  .device-table-panel {
    display: flex;
    flex-direction: column;
    flex: 1 1 auto;
    width: 100%;
    max-width: 100%;
    height: 100%;
    min-height: 0;
    min-width: 0;
    border: 1px solid var(--n-border-color);
    box-shadow: none;
    background: var(--n-color);

    :deep(.n-card-header) {
      flex: 0 0 auto;
      padding: 10px 12px 8px;
    }

    :deep(.n-card-content) {
      display: flex;
      flex: 1;
      flex-direction: column;
      min-height: 0;
      overflow: hidden;
      padding: 0 12px 12px;
    }

    :deep(.n-form) {
      flex: 0 0 auto;
      margin-bottom: 6px;
    }

    :deep(.basic-table) {
      flex: 1 1 auto;
      height: auto;
      min-height: 260px;
    }

    :deep(.s-table) {
      display: flex;
      flex: 1;
      flex-direction: column;
      min-height: 0;
    }

    :deep(.n-data-table) {
      flex: 1 1 auto;
      min-height: 0;
      max-height: 100%;
    }

    :deep(.n-data-table-wrapper) {
      flex: 0 1 auto;
      min-height: 0;
    }

    :deep(.n-data-table-base-table),
    :deep(.n-data-table-base-table-body) {
      min-height: 0;
    }

    :deep(.table-toolbar) {
      align-items: center;
      gap: 6px;
      min-width: 0;
    }

    :deep(.table-toolbar-left) {
      min-width: 0;
      flex: 1 1 auto;
      flex-wrap: wrap;
      gap: 4px;
    }

    :deep(.table-toolbar-right) {
      flex: 0 0 auto;
      white-space: nowrap;
    }

    :deep(.n-data-table__pagination) {
      flex-wrap: wrap;
      margin: 8px 0 0;
      row-gap: 6px;
      background: var(--n-color);
      min-height: 28px;
    }
  }

  .device-table-actions {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
    flex-wrap: wrap;
  }

  .table-header {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 6px;
  }

  .table-header__toggle-row {
    width: 100%;
  }

  .table-header__main {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    width: 100%;
  }

  .table-header__toggle {
    flex: 0 0 auto;
    border-radius: 4px;
    padding: 0 9px;
    background: var(--n-merged-color);
    border: 1px solid var(--n-border-color);
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
    color: var(--n-title-text-color);
    font-size: 14px;
    font-weight: 600;
  }

  .table-header__subtitle {
    margin-top: 2px;
    color: var(--n-text-color-3);
    font-size: 12px;
  }

  .device-action-cell {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 6px;
    padding: 0 2px;
    white-space: nowrap;
  }

  .device-action-cell :deep(.n-button) {
    flex: 0 0 auto;
    margin: 0;
  }

  .device-action-cell__button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    min-width: 26px;
    border-radius: 4px;
  }

  :deep(.device-monitor-empty) {
    padding: 10px 12px;
    color: var(--n-text-color-3);
    font-size: 13px;
    background: var(--n-merged-color);
    border: 1px solid var(--n-border-color);
    border-radius: 4px;
  }

  :deep(.device-monitor-panel) {
    display: flex;
    flex-direction: column;
    gap: 9px;
    width: 100%;
    min-width: 0;
    padding: 10px 12px 12px;
    background: var(--n-color);
    border: 1px solid var(--n-border-color);
    border-radius: 4px;
  }

  :deep(.device-monitor-panel__head) {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    min-width: 0;
    color: var(--n-text-color-2);
  }

  :deep(.device-monitor-panel__title) {
    color: var(--n-title-text-color);
    font-size: 13px;
    font-weight: 700;
    line-height: 20px;
  }

  :deep(.device-monitor-panel__time) {
    color: var(--n-text-color-3);
    font-size: 12px;
    line-height: 18px;
    word-break: break-word;
  }

  :deep(.device-monitor-summary) {
    display: grid;
    grid-template-columns: minmax(360px, 480px) minmax(280px, 380px);
    gap: 16px;
    align-items: start;
    justify-content: start;
    min-width: 0;
  }

  :deep(.device-monitor-bars) {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 7px 12px;
    min-width: 0;
  }

  :deep(.device-monitor-bar) {
    display: grid;
    grid-template-columns: 40px minmax(0, 1fr);
    gap: 7px;
    align-items: center;
    min-width: 0;
  }

  :deep(.device-monitor-bar__label) {
    color: var(--n-text-color-2);
    font-size: 12px;
    font-weight: 600;
    line-height: 18px;
    white-space: nowrap;
  }

  :deep(.device-monitor-bar .n-progress) {
    min-width: 0;
  }

  :deep(.device-monitor-bar .n-progress-graph-line) {
    overflow: hidden;
  }

  :deep(.device-monitor-facts) {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 6px 14px;
    min-width: 0;
  }

  :deep(.device-monitor-fact) {
    display: flex;
    align-items: center;
    min-width: 0;
    gap: 7px;
    color: var(--n-text-color-2);
    font-size: 12px;
    line-height: 18px;
  }

  :deep(.device-monitor-fact span) {
    flex: 0 0 42px;
    color: var(--n-text-color-3);
    font-weight: 600;
    white-space: nowrap;
  }

  :deep(.device-monitor-fact b) {
    min-width: 0;
    color: var(--n-title-text-color);
    font-weight: 600;
    word-break: break-word;
  }

  :deep(.device-monitor-detail) {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 10px 20px;
    min-width: 0;
    padding-top: 9px;
    border-top: 1px solid var(--n-border-color);
  }

  :deep(.device-monitor-detail__group) {
    min-width: 0;
    padding: 0;
  }

  :deep(.device-monitor-detail__title) {
    margin-bottom: 6px;
    color: var(--n-title-text-color);
    font-size: 12px;
    font-weight: 800;
    line-height: 18px;
  }

  :deep(.device-monitor-detail__row) {
    display: grid;
    grid-template-columns: 48px minmax(0, 1fr);
    gap: 7px;
    min-width: 0;
    align-items: start;
    color: var(--n-text-color-2);
    font-size: 12px;
    line-height: 19px;
  }

  :deep(.device-monitor-detail__label) {
    color: var(--n-text-color-3);
    font-weight: 700;
    white-space: nowrap;
  }

  :deep(.device-monitor-detail__value) {
    min-width: 0;
    color: var(--n-title-text-color);
    font-weight: 600;
    overflow-wrap: break-word;
    word-break: normal;
  }

  :deep(.device-monitor-detail__temperature .device-monitor-detail__row) {
    grid-template-columns: 44px minmax(0, 1fr);
  }

  :deep(.device-monitor-detail__temperature .device-monitor-detail__value) {
    color: var(--n-text-color-2);
    font-size: 12px;
    line-height: 19px;
  }

  @media (max-width: 768px) {
    .device-page {
      height: var(--device-page-height, calc(100vh - 82px));
      min-height: 360px;
      overflow: hidden;
    }

    .device-layout,
    .device-layout--collapsed {
      display: flex;
      flex-direction: column;
      gap: 8px;
      height: 100%;
      min-height: 0;
    }

    .device-layout__aside {
      max-height: 46vh;
      overflow: hidden;
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

    .table-header__subtitle {
      display: none;
    }

    .device-action-cell {
      justify-content: flex-start;
      flex-wrap: nowrap;
      gap: 4px;
      padding: 0;
      white-space: nowrap;
    }

    :deep(.device-monitor-summary) {
      gap: 12px;
    }

    :deep(.device-monitor-summary),
    :deep(.device-monitor-bars),
    :deep(.device-monitor-facts),
    :deep(.device-monitor-detail) {
      grid-template-columns: 1fr;
    }

    :deep(.device-monitor-detail) {
      gap: 14px;
    }

    :deep(.device-monitor-panel) {
      padding: 12px;
    }
  }

  @media (max-width: 1180px) {
    .device-layout,
    .device-layout--collapsed {
      display: flex;
      flex-direction: column;
      width: 100%;
      max-width: 100%;
      height: 100%;
      min-height: 0;
      overflow: hidden;
    }

    .device-layout__aside {
      display: flex;
      flex: 0 0 auto;
      width: 100%;
      max-width: 100%;
      max-height: 34vh;
      min-height: 180px;
      overflow: hidden;
    }

    .device-layout--collapsed .device-layout__aside {
      display: none;
    }

    .group-panel {
      width: 100%;
      min-height: 0;
    }

    .device-table-panel {
      width: 100%;
      max-width: 100%;
      min-width: 0;
      flex: 1 1 auto;
      min-height: 0;
    }

    .device-table-panel :deep(.basic-table) {
      min-height: 300px;
    }

    .table-header__subtitle {
      display: none;
    }

    :deep(.n-form) {
      display: grid;
      grid-template-columns: 1fr;
      gap: 6px;
    }

    :deep(.n-form .n-form-item) {
      margin-bottom: 0;
    }

    :deep(.table-toolbar) {
      align-items: flex-start;
      flex-direction: column;
      gap: 6px;
    }

    :deep(.table-toolbar-left),
    :deep(.table-toolbar-right) {
      width: 100%;
      flex-basis: auto;
    }

    :deep(.table-toolbar-right) {
      justify-content: flex-start;
      flex-wrap: wrap;
      white-space: normal;
    }
  }

  @container (max-width: 900px) {
    .device-layout,
    .device-layout--collapsed {
      display: flex;
      flex-direction: column;
      width: 100%;
      max-width: 100%;
      min-height: 0;
      overflow: hidden;
    }

    .device-layout__main,
    .device-table-panel {
      width: 100%;
      max-width: 100%;
      min-width: 0;
    }
  }

  @media (min-width: 769px) and (max-width: 1280px) {
    .device-layout {
      grid-template-columns: 248px minmax(0, 1fr);
    }

    .device-layout--collapsed {
      grid-template-columns: minmax(0, 1fr);
    }

    :deep(.device-monitor-summary) {
      grid-template-columns: 1fr;
      gap: 14px;
    }

    :deep(.device-monitor-detail) {
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: 12px 18px;
    }
  }
</style>
