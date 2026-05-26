<template>
  <RouterView>
    <template #default="{ Component, route }">
      <template v-if="mode === 'production'">
        <transition :name="getTransitionName" appear mode="out-in">
          <keep-alive v-if="shouldKeepAlive(route)" :max="keepAliveMax">
            <component :is="Component" :key="getCacheKey(route)" />
          </keep-alive>
          <component v-else :is="Component" :key="route.fullPath" />
        </transition>
      </template>
      <template v-else>
        <keep-alive v-if="shouldKeepAlive(route)" :max="keepAliveMax">
          <component :is="Component" :key="getCacheKey(route)" />
        </keep-alive>
        <component v-else :is="Component" :key="route.fullPath" />
      </template>
    </template>
  </RouterView>
</template>

<script>
  import { computed, defineComponent, unref } from 'vue';
  import { useProjectSetting } from '@/hooks/setting/useProjectSetting';

  export default defineComponent({
    name: 'MainView',
    components: {},
    props: {
      notNeedKey: {
        type: Boolean,
        default: false,
      },
      animate: {
        type: Boolean,
        default: true,
      },
    },
    setup() {
      const mode = import.meta.env.MODE;
      const { getIsPageAnimate, getPageAnimateType } = useProjectSetting();
      const keepAliveMax = 12;
      const getTransitionName = computed(() => {
        return unref(getIsPageAnimate) ? unref(getPageAnimateType) : '';
      });

      function shouldKeepAlive(route) {
        return Boolean(route?.meta?.keepAlive) && route?.meta?.noKeepAlive !== true;
      }

      function getCacheKey(route) {
        return route?.name || route?.path || route?.fullPath;
      }

      return {
        getTransitionName,
        getCacheKey,
        keepAliveMax,
        mode,
        shouldKeepAlive,
      };
    },
  });
</script>

<style lang="less" scoped></style>
