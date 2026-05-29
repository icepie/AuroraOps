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
  </div>
</template>
<script lang="ts" setup>
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
