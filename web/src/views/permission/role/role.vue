<template>
  <div class="table-page">
    <n-card :bordered="false" class="proCard table-page__card" title="角色管理">
      <BasicTable
        full-height
        :columns="columns"
        :dataSource="dataSource"
        :row-key="(row) => row.id"
        :loading="loading"
        :actionColumn="actionColumn"
        :pagination="false"
        :expanded-row-keys="expandedRowKeys"
        default-expand-all
        @update:expanded-row-keys="handleExpandedRowKeys"
      >
        <template #tableTitle>
          <n-button type="primary" @click="addTable">
            <template #icon>
              <n-icon>
                <PlusOutlined />
              </n-icon>
            </template>
            添加角色
          </n-button>
        </template>
      </BasicTable>
    </n-card>

    <EditRole ref="editRoleRef" @reloadTable="reloadTable" />
    <EditMenuAuth ref="editMenuAuthRef" @reloadTable="reloadTable" />
    <EditDataAuth ref="editDataAuthRef" @reloadTable="reloadTable" />
  </div>
</template>

<script lang="ts" setup>
  import { h, onMounted, reactive, ref } from 'vue';
  import { useDialog, useMessage } from 'naive-ui';
  import { BasicColumn, BasicTable, TableAction } from '@/components/Table';
  import { Delete, getRoleList } from '@/api/system/role';
  import { columns } from './columns';
  import { PlusOutlined } from '@vicons/antd';
  import EditRole from './editRole.vue';
  import EditMenuAuth from './editMenuAuth.vue';
  import EditDataAuth from './editDataAuth.vue';
  import { newState } from '@/views/permission/role/model';

  const message = useMessage();
  const dialog = useDialog();
  const loading = ref(false);
  const dataSource = ref<any>([]);
  const expandedRowKeys = ref<Array<number | string>>([]);
  const editRoleRef = ref();
  const editMenuAuthRef = ref();
  const editDataAuthRef = ref();

  const actionColumn = reactive<BasicColumn>({
    width: 360,
    title: '操作',
    key: 'action',
    fixed: 'right',
    render(record) {
      return h(TableAction, {
        style: 'primary',
        class: 'role-table-action',
        actions: [
          {
            label: '菜单权限',
            onClick: handleMenuAuth.bind(null, record),
            ifShow: () => {
              return record.id !== 1;
            },
            type: 'default',
          },
          {
            label: '数据权限',
            onClick: handleDataAuth.bind(null, record),
            ifShow: () => {
              return record.id !== 1;
            },
            type: 'default',
          },
          {
            label: '添加子角色',
            onClick: handleAddSub.bind(null, record),
          },
          {
            label: '编辑',
            onClick: handleEdit.bind(null, record),
          },
          {
            label: '删除',
            onClick: handleDelete.bind(null, record),
            ifShow: () => {
              return record.id !== 1;
            },
          },
        ],
      });
    },
  });

  function loadDataTable() {
    loading.value = true;
    getRoleList({ pageSize: 100, page: 1 }).then((res) => {
      const list = res.list ?? [];
      dataSource.value = list;
      expandedRowKeys.value = collectRoleKeys(list);
      loading.value = false;
    });
  }

  function collectRoleKeys(list: Recordable[]): Array<number | string> {
    const keys: Array<number | string> = [];
    const walk = (items: Recordable[]) => {
      items.forEach((item) => {
        keys.push(item.id);
        if (Array.isArray(item.children) && item.children.length > 0) {
          walk(item.children);
        }
      });
    };
    walk(list);
    return keys;
  }

  function handleExpandedRowKeys(keys: Array<number | string>) {
    expandedRowKeys.value = keys;
  }

  function reloadTable() {
    loadDataTable();
  }

  function addTable() {
    editRoleRef.value.openModal(null);
  }

  function handleEdit(record: Recordable) {
    editRoleRef.value.openModal(record);
  }

  function handleAddSub(record: Recordable) {
    let state = newState(null);
    state.pid = record.id;
    editRoleRef.value.openModal(state);
  }

  function handleDelete(record: Recordable) {
    dialog.warning({
      title: '警告',
      content: '你确定要删除？',
      positiveText: '确定',
      negativeText: '取消',
      onPositiveClick: () => {
        Delete(record).then((_res) => {
          message.success('操作成功');
          reloadTable();
        });
      },
    });
  }

  async function handleMenuAuth(record: Recordable) {
    editMenuAuthRef.value.openModal(record);
  }

  function handleDataAuth(record: Recordable) {
    editDataAuthRef.value.openModal(record);
  }

  onMounted(() => {
    loadDataTable();
  });
</script>

<style lang="less" scoped>
  .table-page__card :deep(.basic-table) {
    min-height: 280px;
  }

  :deep(.role-table-action > .flex) {
    gap: 4px;
  }

  :deep(.role-table-action .n-button) {
    margin-right: 0 !important;
    margin-left: 0 !important;
  }
</style>
