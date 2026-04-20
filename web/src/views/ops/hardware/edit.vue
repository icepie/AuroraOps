<template>
  <div>
    <n-modal
      v-model:show="showModal"
      :mask-closable="false"
      :show-icon="false"
      preset="dialog"
      transform-origin="center"
      :title="formValue.id > 0 ? '编辑硬件 #' + formValue.id : '新增硬件'"
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
              <n-gi span="1">
                <n-form-item label="所属设备" path="deviceId">
                  <n-select
                    v-model:value="formValue.deviceId"
                    :options="dict.getOptionUnRef(OPS_DEVICE_OPTION_KEY)"
                    filterable
                    clearable
                    placeholder="请选择所属设备"
                  />
                </n-form-item>
              </n-gi>
              <n-gi span="1">
                <n-form-item label="硬件类型" path="assetType">
                  <n-select v-model:value="formValue.assetType" :options="assetTypeOptions" />
                </n-form-item>
              </n-gi>
              <n-gi span="2">
                <n-form-item label="硬件名称" path="assetName">
                  <n-input v-model:value="formValue.assetName" placeholder="请输入硬件名称" />
                </n-form-item>
              </n-gi>
              <n-gi span="1">
                <n-form-item label="品牌" path="brand">
                  <n-input v-model:value="formValue.brand" placeholder="请输入品牌" />
                </n-form-item>
              </n-gi>
              <n-gi span="1">
                <n-form-item label="型号" path="model">
                  <n-input v-model:value="formValue.model" placeholder="请输入型号" />
                </n-form-item>
              </n-gi>
              <n-gi span="2">
                <n-form-item label="序列号" path="serialNo">
                  <n-input v-model:value="formValue.serialNo" placeholder="请输入序列号" />
                </n-form-item>
              </n-gi>
              <n-gi span="2">
                <n-form-item label="规格参数" path="specification">
                  <n-input
                    v-model:value="formValue.specification"
                    type="textarea"
                    placeholder="例如 CPU 型号、内存容量、网卡速率等"
                  />
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
  import { ref, computed } from 'vue';
  import { useDictStore } from '@/store/modules/dict';
  import { Edit, View, MaxSort } from '@/api/opsAsset';
  import { State, newState, rules, assetTypeOptions, loadOptions, OPS_DEVICE_OPTION_KEY } from './model';
  import { useProjectSettingStore } from '@/store/modules/projectSetting';
  import { useMessage } from 'naive-ui';
  import { adaModalWidth } from '@/utils/hotgo';

  const emit = defineEmits(['reloadTable']);
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
            emit('reloadTable');
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

  async function openModal(state: State) {
    showModal.value = true;
    loading.value = true;
    await loadOptions();

    if (!state || state.id < 1) {
      formValue.value = newState(state);
      MaxSort()
        .then((res) => {
          formValue.value.sort = res.sort;
        })
        .finally(() => {
          loading.value = false;
        });
      return;
    }

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
</script>
