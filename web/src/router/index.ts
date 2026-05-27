import { App } from 'vue';
import { createRouter, createWebHashHistory, RouteRecordRaw } from 'vue-router';
import { RedirectRoute } from '@/router/base';
import { PageEnum } from '@/enums/pageEnum';
import { createRouterGuards } from './router-guards';
import { createRouterIcon } from './router-icons';
import { Layout } from '@/router/constant';

// @ts-ignore
const modules = import.meta.glob('./modules/**/*.ts');
const routeModuleList: RouteRecordRaw[] = [];

Object.keys(modules).forEach((key) => {
  const mod = modules[key].default || {};
  const modList = Array.isArray(mod) ? [...mod] : [mod];
  routeModuleList.push(...modList);
});

function sortRoute(a, b) {
  return (a.meta?.sort || 0) - (b.meta?.sort || 0);
}

routeModuleList.sort(sortRoute);

export const RootRoute: RouteRecordRaw = {
  path: '/',
  name: 'Root',
  redirect: PageEnum.BASE_HOME,
  meta: {
    title: 'Root',
  },
};

export const LoginRoute: RouteRecordRaw = {
  path: '/login',
  name: 'Login',
  component: () => import('@/views/login/index.vue'),
  meta: {
    title: '登录',
  },
};

export const OpsTerminalRoute: RouteRecordRaw = {
  path: '/ops/device/terminal',
  name: 'ops_device_terminal_root',
  component: Layout,
  meta: {
    title: '远程终端',
    hidden: true,
    activeMenu: 'opsDevice',
  },
  children: [
    {
      path: '/ops/device/terminal',
      name: 'ops_device_terminal_index',
      component: () => import('@/views/ops/device/terminal.vue'),
      meta: {
        title: '远程终端',
        hidden: true,
        activeMenu: 'opsDevice',
        keepAlive: true,
        cacheKeyByFullPath: true,
      },
    },
  ],
};

export const OpsDesktopRoute: RouteRecordRaw = {
  path: '/ops/device/desktop',
  name: 'ops_device_desktop_root',
  component: Layout,
  meta: {
    title: '远程桌面',
    hidden: true,
    activeMenu: 'opsDevice',
  },
  children: [
    {
      path: '/ops/device/desktop',
      name: 'ops_device_desktop_index',
      component: () => import('@/views/ops/device/desktop.vue'),
      meta: {
        title: '远程桌面',
        hidden: true,
        activeMenu: 'opsDevice',
        noKeepAlive: true,
      },
    },
  ],
};

//需要验证权限
export const asyncRoutes = [...routeModuleList];

//普通路由 无需验证权限
export const constantRouter: any[] = [LoginRoute, RootRoute, RedirectRoute, OpsTerminalRoute, OpsDesktopRoute];

const router = createRouter({
  history: createWebHashHistory(''),
  routes: constantRouter,
  strict: true,
  scrollBehavior: () => ({ left: 0, top: 0 }),
});

export function setupRouter(app: App) {
  app.use(router);
  // 创建路由守卫
  createRouterGuards(router);
  createRouterIcon();
}

export default router;
