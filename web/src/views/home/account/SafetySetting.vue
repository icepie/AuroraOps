<template>
  <n-card :bordered="false" class="security-card">
    <n-list>
      <n-list-item>
        <template #suffix>
          <n-button type="primary" text @click="openUpdatePassForm">修改</n-button>
        </template>
        <n-thing title="登录密码">
          <template #description>
            <span class="text-gray-400">定期更新登录密码，保障运维账号安全。</span>
          </template>
        </n-thing>
      </n-list-item>
    </n-list>
  </n-card>

  <n-modal
    v-model:show="showModal"
    :show-icon="false"
    preset="dialog"
    title="修改登录密码"
    :style="{
      width: dialogWidth,
    }"
  >
    <n-form :label-width="80" :model="formValue" :rules="rules" ref="formRef">
      <n-form-item label="当前密码" path="oldPassword">
        <n-input
          type="password"
          v-model:value="formValue.oldPassword"
          placeholder="请输入当前密码"
        />
      </n-form-item>

      <n-form-item label="新密码" path="newPassword">
        <n-input type="password" v-model:value="formValue.newPassword" placeholder="请输入新密码" />
      </n-form-item>

      <div>
        <n-space justify="end">
          <n-button @click="showModal = false">取消</n-button>
          <n-button type="primary" @click="formSubmit">修改并重新登录</n-button>
        </n-space>
      </div>
    </n-form>
  </n-modal>

</template>

<script lang="ts" setup>
  import { computed, ref } from 'vue';
  import { useMessage } from 'naive-ui';
  import { useRouter, useRoute } from 'vue-router';
  import { adaModalWidth } from '@/utils/hotgo';
  import { updateMemberPwd } from '@/api/system/user';
  import { TABS_ROUTES } from '@/store/mutation-types';
  import { useUserStore } from '@/store/modules/user';

  const userStore = useUserStore();
  const rules = {
    basicName: {
      required: true,
      message: '请输入网站名称',
      trigger: 'blur',
    },
  };

  const formRef: any = ref(null);
  const message = useMessage();
  const router = useRouter();
  const route = useRoute();
  const showModal = ref(false);
  const formValue = ref({
    oldPassword: '',
    newPassword: '',
  });
  const dialogWidth = computed(() => {
    return adaModalWidth(580);
  });

  function formSubmit() {
    formRef.value.validate((errors) => {
      if (!errors) {
        updateMemberPwd({
          oldPassword: formValue.value.oldPassword,
          newPassword: formValue.value.newPassword,
        })
          .then((_res) => {
            message.success('更新成功');

            userStore.logout().then(() => {
              message.success('成功注销登录');
              // 移除标签页
              localStorage.removeItem(TABS_ROUTES);
              router
                .replace({
                  name: 'Login',
                  query: {
                    redirect: route.fullPath,
                  },
                })
                .finally(() => location.reload());
            });
          })
          .finally(() => {
            showModal.value = false;
          });
      } else {
        message.error('验证失败，请填写完整信息');
      }
    });
  }

  function openUpdatePassForm() {
    showModal.value = true;
    formValue.value.newPassword = '';
    formValue.value.oldPassword = '';
  }
</script>

<style lang="less" scoped>
  .security-card {
    margin-top: 8px;
  }
</style>
