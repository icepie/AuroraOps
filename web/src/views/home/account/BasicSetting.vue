<template>
  <div>
    <n-spin :show="show" description="请稍候...">
      <n-card
        v-show="showInfo"
        title="😋 个人信息"
        embedded
        :bordered="false"
        closable
        hoverable
        @close="handleClose"
      >
        <n-row>
          <n-thing content-indented>
            <template #header>
              {{ timeFix() }}，{{ formValue.realName }}，今天又是充满活力的一天！
            </template>
            <template #header-extra> </template>
            <template #description>
              <n-descriptions
                label-placement="left"
                style="margin-top: 15px"
                column="2"
                content-style="padding-right: 20px;"
              >
                <n-descriptions-item label="管理员ID">{{ formValue.id }}</n-descriptions-item>
                <n-descriptions-item label="用户名"> {{ formValue.username }} </n-descriptions-item>
                <!-- 运维版隐藏余额与积分展示 -->
                <!--
                <n-descriptions-item label="余额">{{
                  Number(formValue.balance).toFixed(2)
                }}</n-descriptions-item>
                <n-descriptions-item label="积分">
                  {{ Number(formValue.integral).toFixed(2) }}
                </n-descriptions-item>
                -->
                <n-descriptions-item label="登录IP">{{
                  formValue.lastLoginIp
                }}</n-descriptions-item>
                <n-descriptions-item label="登录时间"
                  >{{ formValue.lastLoginAt }}
                </n-descriptions-item>
                <n-descriptions-item label="累计登录">
                  {{ formValue.loginCount }} 次</n-descriptions-item
                >
                <n-descriptions-item label="注册时间">
                  {{ formValue.createdAt }}
                </n-descriptions-item>
                <n-descriptions-item label="所属部门">
                  <n-tag size="small" type="success" strong round :bordered="false">
                    {{ formValue.deptName }}
                    <template #icon>
                      <n-icon :component="CheckmarkCircle" />
                    </template>
                  </n-tag>
                </n-descriptions-item>
                <n-descriptions-item label="所属角色">
                  <n-tag size="small" type="success" strong round :bordered="false">
                    {{ formValue.roleName }}
                    <template #icon>
                      <n-icon :component="CheckmarkCircle" />
                    </template>
                  </n-tag>
                </n-descriptions-item>
              </n-descriptions>
            </template>
          </n-thing>
        </n-row>
      </n-card>

      <n-form
        :label-width="80"
        :model="formValue"
        :rules="rules"
        ref="formRef"
        style="margin-top: 15px"
      >
        <n-form-item label="头像" path="avatar">
          <FileChooser v-model:value="formValue.avatar" file-type="image" />
        </n-form-item>

        <n-form-item label="姓名" path="realName">
          <n-input v-model:value="formValue.realName" />
        </n-form-item>

        <!-- 运维版隐藏 QQ 号字段 -->
        <!--
        <n-form-item label="QQ号码" path="qq">
          <n-input v-model:value="formValue.qq" placeholder="请输入QQ号码" />
        </n-form-item>
        -->

        <n-form-item label="生日" path="birthday">
          <DatePicker v-model:formValue="formValue.birthday" type="date" />
        </n-form-item>

        <n-form-item label="性别" path="sex">
          <n-radio-group v-model:value="formValue.sex" name="sex">
            <n-space>
              <n-radio :value="1">男</n-radio>
              <n-radio :value="2">女</n-radio>
              <n-radio :value="3">保密</n-radio>
            </n-space>
          </n-radio-group>
        </n-form-item>

        <n-form-item label="所在省市区" path="cityId">
          <CitySelector v-model:value="formValue.cityId" />
        </n-form-item>

        <n-form-item label="联系地址" path="address">
          <n-input type="textarea" v-model:value="formValue.address" placeholder="联系地址" />
        </n-form-item>

        <div>
          <n-space>
            <n-button type="primary" :loading="formBtnLoading" @click="formSubmit"
              >保存更新</n-button
            >
            <n-button :loading="formBtnLoading" @click="resetForm">重置</n-button>
          </n-space>
        </div>
      </n-form>
    </n-spin>
  </div>
</template>

<script lang="ts" setup>
  import { onMounted, ref } from 'vue';
  import { useMessage } from 'naive-ui';
  import CitySelector from '@/components/CitySelector/citySelector.vue';
  import DatePicker from '@/components/DatePicker/datePicker.vue';
  import { getUserInfo, updateMemberProfile } from '@/api/system/user';
  import { CheckmarkCircle } from '@vicons/ionicons5';
  import { timeFix } from '@/utils/hotgo';
  import { UserInfoState, useUserStore } from '@/store/modules/user';
  import FileChooser from '@/components/FileChooser/index.vue';

  const userStore = useUserStore();
  const show = ref(false);
  const formRef: any = ref(null);
  const message = useMessage();
  const formBtnLoading = ref(false);

  const rules = {
    basicName: {
      required: true,
      message: '请输入网站名称',
      trigger: 'blur',
    },
  };

  const formValue = ref<UserInfoState>({
    id: 0,
    deptName: "",
    roleName: "",
    cityLabel: "",
    permissions: [],
    username: "",
    realName: "",
    avatar: "",
    balance: 0,
    sex: 1,
    qq: "",
    email: "",
    mobile: "",
    birthday: "",
    cityId: 0,
    address: "",
    cash: {
      name: "",
      account: "",
      payeeCode: ""
    },
    createdAt: "",
    loginCount: 0,
    lastLoginAt: "",
    lastLoginIp: "",
    integral: 0,
    openId: "",
    inviteCode: ""
  });

  function formSubmit() {
    formBtnLoading.value = true;
    formRef.value.validate((errors) => {
      if (!errors) {
        updateMemberProfile(formValue.value)
          .then((_res) => {
            message.success('更新成功');
            load();
            userStore.GetInfo();
          })
          .finally(() => {
            formBtnLoading.value = false;
          });
      } else {
        message.error('验证失败，请填写完整信息');
      }
    });
  }

  function resetForm() {
    load();
  }

  onMounted(() => {
    load();
  });

  async function load() {
    show.value = true;
    formValue.value = await getUserInfo();
    show.value = false;
  }

  const showInfo = ref(true);
  function handleClose() {
    showInfo.value = false;
  }
</script>
