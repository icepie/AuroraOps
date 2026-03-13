<template>
  <div>
    <div class="n-layout-page-header">
      <n-card :bordered="false" title="设备管理">
        维护运维设备主数据，供资产管理和后续巡检能力复用。
      </n-card>
    </div>
    <n-card :bordered="false" class="proCard">
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
    <Edit ref="editRef" @reloadTable="reloadTable" />
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
  import { PlusOutlined, DeleteOutlined } from '@vicons/antd';
  import { columns, schemas, loadOptions, State } from './model';
  import { adaTableScrollX } from '@/utils/hotgo';
  import Edit from './edit.vue';

  const dict = useDictStore();
  const dialog = useDialog();
  const message = useMessage();
  const { hasPermission } = usePermission();
  const actionRef = ref();
  const searchFormRef = ref<any>({});
  const editRef = ref();
  const checkedIds = ref([]);

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

  const loadDataTable = async (res) => {
    return await List({ ...searchFormRef.value?.formModel, ...res });
  };

  function handleOnCheckedRow(rowKeys) {
    checkedIds.value = rowKeys;
  }

  function reloadTable() {
    actionRef.value?.reload();
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
          reloadTable();
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
          reloadTable();
        });
      },
    });
  }

  function handleStatus(record: Recordable, status: number) {
    Status({ id: record.id, status }).then(() => {
      message.success('设为' + dict.getLabel('sys_normal_disable', status) + '成功');
      reloadTable();
    });
  }

  onMounted(() => {
    loadOptions();
  });
</script>
