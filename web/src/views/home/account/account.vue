<template>
  <div>
    <n-grid cols="24 300:1 600:24" :x-gap="12">
      <n-grid-item span="6">
        <n-card :bordered="false" class="proCard account-nav-card" content-style="padding: 6px 8px;">
          <div
            class="account-nav-item"
            v-for="item in typeTabList"
            :key="item.key"
            :class="{ 'account-nav-item--active': type === item.key }"
            @click="switchType(item)"
          >
            <div class="account-nav-item__title">{{ item.name }}</div>
            <div class="account-nav-item__desc">{{ item.desc }}</div>
          </div>
        </n-card>
      </n-grid-item>
      <n-grid-item span="18">
        <n-card :bordered="false" size="small" :title="typeTitle" class="proCard">
          <BasicSetting v-if="type === 1" />
          <SafetySetting v-if="type === 2" />
          <!-- 运维版暂不开放提现设置 -->
          <!-- <CashSetting v-if="type === 3" /> -->
          <!-- 运维版暂不开放第三方绑定 -->
          <!-- <ThirdBind v-if="type === 4" /> -->
        </n-card>
      </n-grid-item>
    </n-grid>
  </div>
</template>
<script lang="ts" setup>
  import { ref, onMounted } from 'vue';
  import BasicSetting from './BasicSetting.vue';
  import SafetySetting from './SafetySetting.vue';
  // import CashSetting from './CashSetting.vue';
  // import ThirdBind from './ThirdBind.vue';
  import { useRouter } from 'vue-router';
  import { pushHashRouterParameter } from '@/utils/urlUtils';

  const router = useRouter();
  const type = ref(1);
  const typeTitle = ref('基本设置');

  const typeTabList = [
    {
      name: '基本设置',
      desc: '个人账户信息设置',
      key: 1,
    },
    {
      name: '安全设置',
      desc: '密码、手机号、邮箱等设置',
      key: 2,
    },
    // {
    //   name: '提现设置',
    //   desc: '提现收款账号支付宝设置',
    //   key: 3,
    // },
    // {
    //   name: '第三方绑定',
    //   desc: '第三方快捷登录、消息推送',
    //   key: 4,
    // },
  ];

  onMounted(() => {
    if (router.currentRoute.value.query?.type) {
      setDefaultOption();
    }
  });

  function setDefaultOption() {
    const key = router.currentRoute.value.query.type as unknown as number;
    if (key !== undefined && key > 0) {
      for (const item of typeTabList) {
        if (item.key == key) {
          switchType(item);
        }
      }
    }
  }

  function switchType(e) {
    type.value = e.key;
    typeTitle.value = e.name;
    pushHashRouterParameter(window.location.href, 'type', e.key);
  }
</script>
<style lang="less" scoped>
  .account-nav-card {
    min-height: 100%;
  }

  .account-nav-item {
    padding: 6px 8px;
    border-radius: 4px;
    cursor: pointer;
    transition: background-color 0.2s, color 0.2s;

    &:hover {
      background: #f3f3f3;
    }
  }

  .account-nav-item + .account-nav-item {
    margin-top: 2px;
  }

  .account-nav-item__title {
    font-size: 13px;
    line-height: 18px;
    font-weight: 600;
  }

  .account-nav-item__desc {
    margin-top: 1px;
    font-size: 11px;
    line-height: 16px;
    color: #64748b;
  }

  .account-nav-item--active {
    background: #f0faff;
    color: #2d8cf0;

    .account-nav-item__title,
    .account-nav-item__desc {
      color: #2d8cf0;
    }

    &:hover {
      background: #f0faff;
    }
  }
</style>
