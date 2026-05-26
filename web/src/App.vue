<template>
  <NConfigProvider
    v-if="!shouldShowLockScreen"
    :locale="zhCN"
    :theme="getDarkTheme"
    :theme-overrides="getThemeOverrides"
    :date-locale="dateZhCN"
  >
    <AppProvider>
      <RouterView />
    </AppProvider>
  </NConfigProvider>

  <transition v-if="shouldShowLockScreen" name="slide-up">
    <LockScreen />
  </transition>
</template>

<script lang="ts" setup>
  import { computed, onMounted, onUnmounted } from 'vue';
  import { zhCN, dateZhCN, darkTheme } from 'naive-ui';
  import { LockScreen } from '@/components/Lockscreen';
  import { AppProvider } from '@/components/Application';
  import { useLockscreenStore } from '@/store/modules/lockscreen';
  import { useRoute } from 'vue-router';
  import { useDesignSettingStore } from '@/store/modules/designSetting';
  import { lighten } from '@/utils';

  const route = useRoute();
  const useLockscreen = useLockscreenStore();
  const designStore = useDesignSettingStore();
  const isLock = computed(() => useLockscreen.isLock);
  const lockTime = computed(() => useLockscreen.lockTime);
  const remoteSessionRoutes = new Set(['/ops/device/desktop', '/ops/device/terminal']);
  const isRemoteSessionRoute = computed(() => remoteSessionRoutes.has(route.path));
  const shouldShowLockScreen = computed(
    () => isLock.value && route.name !== 'login' && !isRemoteSessionRoute.value
  );

  /**
   * @type import('naive-ui').GlobalThemeOverrides
   */
  const getThemeOverrides = computed(() => {
    const appTheme = designStore.appTheme;
    const lightenStr = lighten(designStore.appTheme, 6);
    return {
      common: {
        primaryColor: appTheme,
        primaryColorHover: lightenStr,
        primaryColorPressed: lightenStr,
        fontSize: '13px',
        fontSizeMini: '11px',
        fontSizeTiny: '12px',
        fontSizeSmall: '12px',
        fontSizeMedium: '13px',
        fontSizeLarge: '14px',
        heightTiny: '20px',
        heightSmall: '24px',
        heightMedium: '28px',
        heightLarge: '32px',
        borderRadius: '5px',
        // 纵向滚动条宽
        scrollbarWidth: '6px',
        // 横向滚动条高
        scrollbarHeight: '6px',
      },
      Button: {
        paddingTiny: '0 6px',
        paddingSmall: '0 8px',
        paddingMedium: '0 10px',
        iconMarginSmall: '4px',
        iconMarginMedium: '5px',
      },
      Card: {
        titleFontSizeSmall: '14px',
        titleFontSizeMedium: '14px',
        paddingSmall: '8px 10px',
        paddingMedium: '9px 12px',
      },
      DataTable: {
        fontSizeSmall: '12px',
        fontSizeMedium: '13px',
        thPaddingSmall: '5px 8px',
        tdPaddingSmall: '4px 8px',
        thPaddingMedium: '6px 9px',
        tdPaddingMedium: '5px 9px',
      },
      Form: {
        labelFontSizeLeftMedium: '12px',
        labelFontSizeTopMedium: '12px',
        feedbackHeightMedium: '16px',
      },
      LoadingBar: {
        colorLoading: appTheme,
      },
    };
  });

  const getDarkTheme = computed(() => (designStore.darkTheme ? darkTheme : undefined));

  let timer;

  const timekeeping = () => {
    clearInterval(timer);
    if (route.name == 'login') return;
    // 设置不锁屏
    useLockscreen.setLock(false);
    if (isRemoteSessionRoute.value) return;
    // 重置锁屏时间
    useLockscreen.setLockTime();
    timer = setInterval(() => {
      // 锁屏倒计时递减
      useLockscreen.setLockTime(lockTime.value - 1);
      if (lockTime.value <= 0) {
        // 设置锁屏
        useLockscreen.setLock(true);
        return clearInterval(timer);
      }
    }, 1000);
  };

  onMounted(() => {
    document.addEventListener('mousedown', timekeeping);
  });

  onUnmounted(() => {
    document.removeEventListener('mousedown', timekeeping);
  });
</script>

<style lang="less">
  @import 'styles/index.less';
</style>
