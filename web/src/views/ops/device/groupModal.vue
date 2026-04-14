<template>
  <n-modal
    v-model:show="showModal"
    :mask-closable="false"
    :show-icon="false"
    preset="dialog"
    transform-origin="center"
    :title="formValue.id ? '编辑分组 #' + formValue.id : '新增设备分组'"
    :style="{ width: dialogWidth }"
  >
    <n-spin :show="loading" description="请稍候...">
      <n-form
        ref="formRef"
        :model="formValue"
        :rules="rules"
        :label-placement="settingStore.isMobile ? 'top' : 'left'"
        :label-width="96"
        class="py-4"
      >
        <n-grid cols="1 s:1 m:2 l:2 xl:2 2xl:2" responsive="screen">
          <n-gi span="2">
            <n-form-item label="分组名称" path="name">
              <n-input v-model:value="formValue.name" placeholder="请输入分组名称" />
            </n-form-item>
          </n-gi>
          <n-gi span="1">
            <n-form-item label="排序" path="sort">
              <n-input-number v-model:value="formValue.sort" placeholder="请输入排序" />
            </n-form-item>
          </n-gi>
          <n-gi span="1">
            <n-form-item label="状态" path="status">
              <n-select
                v-model:value="formValue.status"
                :options="dict.getOptionUnRef('sys_normal_disable')"
              />
            </n-form-item>
          </n-gi>
          <n-gi span="2">
            <n-form-item label="备注" path="remark">
              <n-input v-model:value="formValue.remark" type="textarea" placeholder="请输入备注" />
            </n-form-item>
          </n-gi>
        </n-grid>
      </n-form>
    </n-spin>
    <template #action>
      <n-space>
        <n-button @click="closeForm">取消</n-button>
        <n-button type="info" :loading="formBtnLoading" @click="confirmForm">确定</n-button>
      </n-space>
    </template>
  </n-modal>
</template>

<script lang="ts" setup>
  import { computed, ref } from 'vue';
  import { useDictStore } from '@/store/modules/dict';
  import { useProjectSettingStore } from '@/store/modules/projectSetting';
  import { useMessage } from 'naive-ui';
  import { adaModalWidth } from '@/utils/hotgo';
  import { Edit, MaxSort, View } from '@/api/opsDeviceGroup';

  const emit = defineEmits(['reload-groups']);
  const dict = useDictStore();
  const message = useMessage();
  const settingStore = useProjectSettingStore();
  const showModal = ref(false);
  const loading = ref(false);
  const formBtnLoading = ref(false);
  const formRef = ref<any>({});
  const dialogWidth = computed(() => adaModalWidth(640));

  const formValue = ref({
    id: 0,
    name: '',
    sort: 0,
    status: 1,
    remark: '',
  });

  const rules = {
    name: {
      required: true,
      trigger: ['blur', 'input'],
      message: '请输入分组名称',
    },
    sort: {
      required: true,
      trigger: ['blur', 'input'],
      type: 'number',
      message: '请输入排序',
    },
  };

  function resetForm() {
    formValue.value = {
      id: 0,
      name: '',
      sort: 0,
      status: 1,
      remark: '',
    };
  }

  function closeForm() {
    showModal.value = false;
    loading.value = false;
  }

  function confirmForm(e) {
    e.preventDefault();
    formRef.value.validate((errors) => {
      if (!errors) {
        formBtnLoading.value = true;
        Edit(formValue.value)
          .then(() => {
            message.success('操作成功');
            closeForm();
            emit('reload-groups');
          })
          .finally(() => {
            formBtnLoading.value = false;
          });
      } else {
        message.error('请填写完整信息');
      }
    });
  }

  function openModal(record?: { id?: number } | null) {
    showModal.value = true;
    resetForm();

    if (!record?.id) {
      loading.value = true;
      MaxSort()
        .then((res) => {
          formValue.value.sort = res.sort;
        })
        .finally(() => {
          loading.value = false;
        });
      return;
    }

    loading.value = true;
    View({ id: record.id })
      .then((res) => {
        formValue.value = {
          id: res.id,
          name: res.name,
          sort: res.sort,
          status: res.status,
          remark: res.remark,
        };
      })
      .finally(() => {
        loading.value = false;
      });
  }

  defineExpose({
    openModal,
  });
</script>
