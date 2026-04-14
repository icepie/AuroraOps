import { http } from '@/utils/http/axios';

export function List() {
  return http.request({
    url: '/opsDeviceGroup/list',
    method: 'GET',
  });
}

export function Delete(params) {
  return http.request({
    url: '/opsDeviceGroup/delete',
    method: 'POST',
    params,
  });
}

export function Edit(params) {
  return http.request({
    url: '/opsDeviceGroup/edit',
    method: 'POST',
    params,
  });
}

export function Status(params) {
  return http.request({
    url: '/opsDeviceGroup/status',
    method: 'POST',
    params,
  });
}

export function View(params) {
  return http.request({
    url: '/opsDeviceGroup/view',
    method: 'GET',
    params,
  });
}

export function MaxSort() {
  return http.request({
    url: '/opsDeviceGroup/maxSort',
    method: 'GET',
  });
}

export function Option() {
  return http.request({
    url: '/opsDeviceGroup/option',
    method: 'GET',
  });
}
