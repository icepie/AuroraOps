<template>
  <RouterView>
    <template #default="{ Component, route }">
      <template v-if="mode === 'production'">
        <transition :name="getTransitionName" appear>
          <keep-alive :max="keepAliveMax">
            <component
              v-if="shouldKeepAlive(route)"
              :is="Component"
              :key="getCacheKey(route)"
            />
          </keep-alive>
        </transition>
        <transition :name="getTransitionName" appear mode="out-in">
          <component v-if="!shouldKeepAlive(route)" :is="Component" :key="route.fullPath" />
        </transition>
      </template>
      <template v-else>
        <keep-alive :max="keepAliveMax">
          <component
            v-if="shouldKeepAlive(route)"
            :is="Component"
            :key="getCacheKey(route)"
          />
        </keep-alive>
        <component v-if="!shouldKeepAlive(route)" :is="Component" :key="route.fullPath" />
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
      const keepAliveMax = 32;
      const getTransitionName = computed(() => {
        return unref(getIsPageAnimate) ? unref(getPageAnimateType) : '';
      });

      function shouldKeepAlive(route) {
        return Boolean(route?.meta?.keepAlive) && route?.meta?.noKeepAlive !== true;
      }

      function getCacheKey(route) {
        if (route?.meta?.cacheKeyByFullPath) {
          return route.fullPath;
        }
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
