import { adminMenus } from '@/api/system/menu';
import { constantRouterIcon } from './router-icons';
import { RouteRecordRaw } from 'vue-router';
import { Layout, ParentLayout } from '@/router/constant';
import type { AppRouteRecordRaw } from '@/router/types';

const Iframe = () => import('@/views/iframe/index.vue');
const LayoutMap = new Map<string, () => Promise<typeof import('*.vue')>>();
const OPS_ROUTE_NAME_BLACKLIST = new Set([
  'dashboard_workplace',
  'apidocs',
  'about',
  'about_index',
  'addons',
  'hgexample',
  'hgexample_portal',
  'hgexample_config',
  'hgexample_table_index',
  'table_view',
  'develop_code',
  'develop_code_deploy',
  'develop_addons',
  'apply_notice',
  'apply_attachment',
  'apply_provinces',
  'home',
  'home_account',
  'home_message',
  'asset',
  'asset_recharge',
  'asset_cash',
  'creditsLogIndex',
  'sms_log',
]);
const OPS_ROUTE_PATH_BLACKLIST = ['/doc', '/about', '/addons', '/develop'];
const OPS_ROUTE_PATH_FRAGMENT_BLACKLIST = [
  '/workplace',
  '/hgexample',
  '/asset',
  '/home',
  '/addons',
  '/develop',
];
const OPS_ROUTE_COMPONENT_FRAGMENT_BLACKLIST = [
  '/dashboard/workplace/',
  '/addons/',
  '/develop/',
  '/asset/',
  '/home/',
  '/about/',
  '/apply/notice/',
  '/apply/attachment/',
  '/apply/provinces/',
];
const OPS_ROUTE_TITLE_MAP: Record<string, string> = {
  Dashboard: '运维总览',
  dashboard_console: '运维总览',
  Applys: '日志中心',
};
const OPS_HIDDEN_ROUTE_NAMES = new Set(['home', 'home_account']);

LayoutMap.set('LAYOUT', Layout);
LayoutMap.set('IFRAME', Iframe);

function shouldHideOpsRoute(route: any) {
  const routeName = route.name || '';
  const routePath = route.path || '';
  const routeComponent = typeof route.component === 'string' ? route.component : '';

  if (OPS_HIDDEN_ROUTE_NAMES.has(routeName)) {
    return false;
  }

  if (OPS_ROUTE_NAME_BLACKLIST.has(routeName)) {
    return true;
  }

  if (OPS_ROUTE_PATH_BLACKLIST.includes(routePath)) {
    return true;
  }

  if (OPS_ROUTE_PATH_FRAGMENT_BLACKLIST.some((item) => routePath.includes(item))) {
    return true;
  }

  if (
    OPS_ROUTE_COMPONENT_FRAGMENT_BLACKLIST.some((item) => routeComponent.includes(item))
  ) {
    return true;
  }

  return false;
}

function normalizeOpsRouteMeta(route: any) {
  const title = OPS_ROUTE_TITLE_MAP[route.name];
  route.meta = {
    ...route.meta,
    ...(title
      ? {
          title,
          label: title,
        }
      : {}),
    hidden: OPS_HIDDEN_ROUTE_NAMES.has(route.name) ? true : route.meta?.hidden,
  };

  const isVisibleMenuPage = String(route.meta?.type) === '2' && route.meta?.hidden !== true;
  if (isVisibleMenuPage && route.meta?.activeMenu && route.meta.activeMenu !== route.name) {
    delete route.meta.activeMenu;
  }

  return route;
}

/**
 * 格式化 后端 结构信息并递归生成层级路由表
 * @param routerMap
 * @param parent
 * @returns {*}
 */
export const routerGenerator = (routerMap, parent?): any[] => {
  return routerMap
    .map((item) => {
      const currentRouter: any = {
        // 路由地址 动态拼接生成如 /dashboard/workplace
        path: `${(parent && parent.path) || ''}/${item.path}`,
        // 路由名称，建议唯一
        name: item.name || '',
        // 该路由对应页面的 组件
        component: item.component,
        // meta: 页面标题, 菜单图标, 页面权限(供指令权限用，可去掉)
        meta: {
          ...item.meta,
          label: item.meta.title,
          icon: constantRouterIcon[item.meta.icon] || null,
          permissions: item.meta.permissions || null,
        },
      };

      // 为了防止出现后端返回结果不规范，处理有可能出现拼接出两个 反斜杠
      currentRouter.path = currentRouter.path.replace('//', '/');
      // 重定向 ,菜单类型为目录默认默认跳转
      if (item.meta.type === 1) {
        item.redirect && (currentRouter.redirect = item.redirect);
      }
      // 是否有子菜单，并递归处理
      if (item.children && item.children.length > 0) {
        //如果未定义 redirect 默认第一个子路由为 redirect
        if (item.meta.type === 1) {
          !item.redirect && (currentRouter.redirect = `${item.path}/${item.children[0].path}`);
        }
        // Recursion
        currentRouter.children = routerGenerator(item.children, currentRouter);
      }

      normalizeOpsRouteMeta(currentRouter);
      return currentRouter;
    })
    .filter((route) => {
      if (shouldHideOpsRoute(route)) {
        return false;
      }

      if (route.children && route.children.length === 0) {
        delete route.children;
      }

      if (route.meta?.type === 1 && !route.children?.length && route.component === 'LAYOUT') {
        return false;
      }

      return true;
    });
};

/**
 * 动态生成菜单
 */
export const generatorDynamicRouter = (): Promise<RouteRecordRaw[]> => {
  return new Promise((resolve, reject) => {
    adminMenus()
      .then((result) => {
        const routeList = routerGenerator(result.list);
        asyncImportRoute(routeList);

        resolve(routeList);
      })
      .catch((err) => {
        reject(err);
      });
  });
};

/**
 * 查找views中对应的组件文件
 * */
let viewsModules: Record<string, () => Promise<Recordable>>;
export const asyncImportRoute = (routes: AppRouteRecordRaw[] | undefined): void => {
  viewsModules = viewsModules || import.meta.glob('../views/**/*.{vue,tsx}');
  if (!routes) return;
  routes.forEach((item) => {
    if (!item.component && item.meta?.frameSrc) {
      item.component = 'IFRAME';
    }
    const { component, name } = item;
    const { children } = item;
    if (component) {
      const layoutFound = LayoutMap.get(component as string);
      if (layoutFound) {
        item.component = layoutFound;
      } else {
        item.component = dynamicImport(viewsModules, component as string);
      }
    } else if (name) {
      item.component = ParentLayout;
    }
    children && asyncImportRoute(children);
  });
};

/**
 * 动态导入
 * */
export const dynamicImport = (
  viewsModules: Record<string, () => Promise<Recordable>>,
  component: string
) => {
  const keys = Object.keys(viewsModules);
  const matchKeys = keys.filter((key) => {
    let k = key.replace('../views', '');
    const lastIndex = k.lastIndexOf('.');
    k = k.substring(0, lastIndex);
    return k === component;
  });
  if (matchKeys?.length === 1) {
    const matchKey = matchKeys[0];
    return viewsModules[matchKey];
  }
  if (matchKeys?.length > 1) {
    console.warn(
      'Please do not create `.vue` and `.TSX` files with the same file name in the same hierarchical directory under the views folder. This will cause dynamic introduction failure'
    );
    return;
  }
};

/**
 * 移除隐藏的菜单
 * @param menus
 */
export const removeHiddenMenus = (menus: any[]) => {
  const arr: any[] = [];
  for (let j = 0; j < menus.length; j++) {
    if (menus[j].meta?.type === 3) {
      continue;
    }
    if (menus[j].meta?.hidden === true) {
      continue;
    }

    if (menus[j].children?.length > 0) {
      menus[j].children = removeHiddenMenus(menus[j].children);
      if (menus[j].children?.length === 0) {
        delete menus[j].children;
      }
    }
    arr.push(menus[j]);
  }
  return arr;
};
