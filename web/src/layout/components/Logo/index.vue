<template>
  <div class="logo">
    <img :src="siteLogo" alt="" />
    <h2 v-show="!collapsed" class="title">{{ projectName }}</h2>
  </div>
</template>

<script lang="ts">
  import { computed, defineComponent } from 'vue';
  import { useUserStore } from '@/store/modules/user';

  export default defineComponent({
    name: 'Index',
    props: {
      collapsed: {
        type: Boolean,
      },
    },
    setup() {
      const userStore = useUserStore();
      const projectName = computed(() => userStore.getSiteName);
      const siteLogo = computed(() => userStore.getSiteLogo);
      return {
        projectName,
        siteLogo,
      };
    },
  });
</script>

<style lang="less" scoped>
  .logo {
    display: flex;
    align-items: center;
    justify-content: flex-start;
    gap: 8px;
    box-sizing: border-box;
    height: @header-height;
    overflow: hidden;
    padding: 0 12px;
    white-space: nowrap;

    img {
      flex: 0 0 auto;
      width: 24px;
      height: 24px;
      display: block;
      object-fit: contain;
      border-radius: 4px;
    }

    .title {
      display: inline-flex;
      align-items: center;
      min-width: 0;
      margin-bottom: 0;
      overflow: hidden;
      height: 24px;
      font-size: 14px;
      font-weight: 600;
      line-height: 24px;
      letter-spacing: 0;
      text-overflow: ellipsis;
    }
  }
</style>
