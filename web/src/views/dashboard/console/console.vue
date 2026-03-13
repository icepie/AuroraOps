<template>
  <div class="ops-console">
    <n-grid cols="1 s:2 l:4" responsive="screen" :x-gap="16" :y-gap="16">
      <n-grid-item v-for="item in quickEntries" :key="item.path">
        <n-card :bordered="false" hoverable>
          <template #header>
            <div class="card-header">
              <span>{{ item.title }}</span>
              <n-tag size="small" type="primary" :bordered="false">{{ item.tag }}</n-tag>
            </div>
          </template>
          <n-text depth="3">{{ item.description }}</n-text>
          <template #footer>
            <n-button text type="primary" @click="router.push(item.path)">进入模块</n-button>
          </template>
        </n-card>
      </n-grid-item>
    </n-grid>

    <n-grid class="mt-4" cols="1 l:2" responsive="screen" :x-gap="16" :y-gap="16">
      <n-grid-item>
        <n-card title="当前运维账号" :bordered="false">
          <n-descriptions label-placement="left" :column="1">
            <n-descriptions-item label="账号">{{ userStore.info?.username || '-' }}</n-descriptions-item>
            <n-descriptions-item label="姓名">{{ userStore.info?.realName || '-' }}</n-descriptions-item>
            <n-descriptions-item label="角色">{{ userStore.info?.roleName || '-' }}</n-descriptions-item>
            <n-descriptions-item label="部门">{{ userStore.info?.deptName || '-' }}</n-descriptions-item>
            <n-descriptions-item label="最近登录">{{ userStore.info?.lastLoginAt || '-' }}</n-descriptions-item>
            <n-descriptions-item label="最近 IP">{{ userStore.info?.lastLoginIp || '-' }}</n-descriptions-item>
          </n-descriptions>
        </n-card>
      </n-grid-item>
      <n-grid-item>
        <n-card title="交付说明" :bordered="false">
          <n-space vertical :size="12">
            <n-alert type="success" :show-icon="false">
              当前前端已屏蔽默认演示、插件、多租户、多商户、资金和文档类菜单。
            </n-alert>
            <n-alert type="info" :show-icon="false">
              登录页仅保留账号密码登录，演示账号、手机号登录、注册入口已注释隐藏。
            </n-alert>
            <n-alert type="warning" :show-icon="false">
              如需继续收敛后台接口或数据库菜单，我可以下一步再做后端侧清理。
            </n-alert>
          </n-space>
        </n-card>
      </n-grid-item>
    </n-grid>
  </div>
</template>
<script lang="ts" setup>
  import { useUserStore } from '@/store/modules/user';

  const userStore = useUserStore();
  const router = useRouter();
  const quickEntries = [
    {
      title: '人员管理',
      tag: '组织',
      description: '维护后台运维账号、部门和岗位信息。',
      path: '/org/user',
    },
    {
      title: '权限管理',
      tag: '权限',
      description: '配置角色授权、菜单权限和访问范围。',
      path: '/permission/menu',
    },
    {
      title: '系统配置',
      tag: '配置',
      description: '维护基础配置、黑名单和系统参数。',
      path: '/system/config',
    },
    {
      title: '系统监控',
      tag: '监控',
      description: '查看服务状态、在线会话和运行监控信息。',
      path: '/monitor/serve_monitor',
    },
    {
      title: '运维日志',
      tag: '日志',
      description: '审计登录记录、服务日志和关键操作。',
      path: '/apply/log/log',
    },
  ];
</script>
<style lang="less" scoped>
  .ops-console {
    margin-top: 8px;
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
</style>
