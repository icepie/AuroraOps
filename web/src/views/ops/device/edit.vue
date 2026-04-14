<template>
  <div>
    <n-modal
      v-model:show="showModal"
      :mask-closable="false"
      :show-icon="false"
      preset="dialog"
      transform-origin="center"
      :title="formValue.id > 0 ? '编辑设备 #' + formValue.id : '新增设备'"
      :style="{ width: dialogWidth }"
    >
      <n-scrollbar style="max-height: 87vh" class="pr-5">
        <n-spin :show="loading" description="请稍候...">
          <n-form
            ref="formRef"
            :model="formValue"
            :rules="rules"
            :label-placement="settingStore.isMobile ? 'top' : 'left'"
            :label-width="100"
            class="py-4"
          >
            <n-grid cols="1 s:1 m:2 l:2 xl:2 2xl:2" responsive="screen">
              <n-gi span="2">
                <n-form-item label="设备分组" path="groupId">
                  <n-select
                    v-model:value="formValue.groupId"
                    :options="dict.getOptionUnRef(OPS_DEVICE_GROUP_OPTION_KEY)"
                    clearable
                    filterable
                    placeholder="请选择设备分组"
                  />
                </n-form-item>
              </n-gi>
              <n-gi span="2">
                <n-form-item label="设备名称" path="name">
                  <n-input v-model:value="formValue.name" placeholder="请输入设备名称" />
                </n-form-item>
              </n-gi>
              <n-gi span="1">
                <n-form-item label="主机名" path="hostname">
                  <n-input v-model:value="formValue.hostname" placeholder="请输入主机名" />
                </n-form-item>
              </n-gi>
              <n-gi span="1">
                <n-form-item label="IP地址" path="ip">
                  <n-input v-model:value="formValue.ip" placeholder="请输入IP地址" />
                </n-form-item>
              </n-gi>
              <n-gi span="1">
                <n-form-item label="设备类型" path="deviceType">
                  <n-select v-model:value="formValue.deviceType" :options="deviceTypeOptions" />
                </n-form-item>
              </n-gi>
              <n-gi span="1">
                <n-form-item label="操作系统" path="osName">
                  <n-input v-model:value="formValue.osName" placeholder="如 Ubuntu 24.04" />
                </n-form-item>
              </n-gi>
              <n-gi span="2">
                <n-form-item label="部署位置" path="location">
                  <n-input v-model:value="formValue.location" placeholder="请输入部署位置" />
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
                  <n-input
                    v-model:value="formValue.remark"
                    type="textarea"
                    placeholder="请输入备注"
                  />
                </n-form-item>
              </n-gi>
            </n-grid>
          </n-form>
        </n-spin>
      </n-scrollbar>
      <template #action>
        <n-space>
          <n-button @click="closeForm">取消</n-button>
          <n-button type="info" :loading="formBtnLoading" @click="confirmForm">确定</n-button>
        </n-space>
      </template>
    </n-modal>
  </div>
</template>

<script lang="ts" setup>
  import { ref, computed, onMounted } from 'vue';
  import { useDictStore } from '@/store/modules/dict';
  import { Edit, View, MaxSort } from '@/api/opsDevice';
  import {
    State,
    newState,
    rules,
    deviceTypeOptions,
    OPS_DEVICE_GROUP_OPTION_KEY,
    loadGroupOptions,
  } from './model';
  import { useProjectSettingStore } from '@/store/modules/projectSetting';
  import { useMessage } from 'naive-ui';
  import { adaModalWidth } from '@/utils/hotgo';

  const emit = defineEmits(['reload-table']);
  const message = useMessage();
  const settingStore = useProjectSettingStore();
  const dict = useDictStore();
  const loading = ref(false);
  const showModal = ref(false);
  const formValue = ref<State>(newState(null));
  const formRef = ref<any>({});
  const formBtnLoading = ref(false);
  const dialogWidth = computed(() => adaModalWidth(840));

  function confirmForm(e) {
    e.preventDefault();
    formRef.value.validate((errors) => {
      if (!errors) {
        formBtnLoading.value = true;
        Edit(formValue.value)
          .then(() => {
            message.success('操作成功');
            closeForm();
            emit('reload-table');
          })
          .finally(() => {
            formBtnLoading.value = false;
          });
      } else {
        message.error('请填写完整信息');
      }
    });
  }

  function closeForm() {
    showModal.value = false;
    loading.value = false;
  }

  function openModal(state: State) {
    showModal.value = true;

    if (!state || state.id < 1) {
      formValue.value = newState(state);
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
    View({ id: state.id })
      .then((res) => {
        formValue.value = newState(res);
      })
      .finally(() => {
        loading.value = false;
      });
  }

  defineExpose({
    openModal,
  });

  onMounted(() => {
    loadGroupOptions();
  });
</script>
