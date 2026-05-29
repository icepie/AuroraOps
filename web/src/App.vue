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
  import { computed, onMounted, onUnmounted, watch } from 'vue';
  import { zhCN, dateZhCN, darkTheme } from 'naive-ui';
  import { LockScreen } from '@/components/Lockscreen';
  import { AppProvider } from '@/components/Application';
  import { useLockscreenStore } from '@/store/modules/lockscreen';
  import { useRoute } from 'vue-router';
  import { useDesignSettingStore } from '@/store/modules/designSetting';
  import { useProjectSettingStore } from '@/store/modules/projectSetting';
  import { lighten } from '@/utils';

  const route = useRoute();
  const useLockscreen = useLockscreenStore();
  const designStore = useDesignSettingStore();
  const projectSettingStore = useProjectSettingStore();
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
        bodyColor: designStore.darkTheme ? '#0f1115' : '#f5f7fa',
        cardColor: designStore.darkTheme ? '#171a21' : '#ffffff',
        modalColor: designStore.darkTheme ? '#171a21' : '#ffffff',
        popoverColor: designStore.darkTheme ? '#1b1f27' : '#ffffff',
        tableColor: designStore.darkTheme ? '#171a21' : '#ffffff',
        tableHeaderColor: designStore.darkTheme ? '#20242d' : '#f7f8fa',
        borderColor: designStore.darkTheme ? 'rgba(255,255,255,0.10)' : '#e8eaec',
        dividerColor: designStore.darkTheme ? 'rgba(255,255,255,0.09)' : '#edf0f5',
        textColorBase: designStore.darkTheme ? '#eef2f8' : '#111827',
        textColor1: designStore.darkTheme ? '#f8fafc' : '#111827',
        textColor2: designStore.darkTheme ? '#e2e8f0' : '#1f2937',
        textColor3: designStore.darkTheme ? '#cbd5e1' : '#374151',
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
        borderRadius: '4px',
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
        paddingSmall: '7px 10px',
        paddingMedium: '8px 12px',
      },
      DataTable: {
        fontSizeSmall: '12px',
        fontSizeMedium: '13px',
        thPaddingSmall: '4px 8px',
        tdPaddingSmall: '3px 8px',
        thPaddingMedium: '5px 9px',
        tdPaddingMedium: '4px 9px',
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

  watch(
    () => designStore.darkTheme,
    (value) => {
      document.documentElement.dataset.theme = value ? 'dark' : 'light';
      projectSettingStore.navTheme = value ? 'header-dark' : 'light';
    },
    { immediate: true }
  );

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
