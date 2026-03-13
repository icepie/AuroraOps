import { http } from '@/utils/http/axios';

export function List(params) {
  return http.request({
    url: '/opsAsset/list',
    method: 'get',
    params,
  });
}

export function Delete(params) {
  return http.request({
    url: '/opsAsset/delete',
    method: 'POST',
    params,
  });
}

export function Edit(params) {
  return http.request({
    url: '/opsAsset/edit',
    method: 'POST',
    params,
  });
}

export function Status(params) {
  return http.request({
    url: '/opsAsset/status',
    method: 'POST',
    params,
  });
}

export function View(params) {
  return http.request({
    url: '/opsAsset/view',
    method: 'GET',
    params,
  });
}

export function MaxSort() {
  return http.request({
    url: '/opsAsset/maxSort',
    method: 'GET',
  });
}
