import { http } from '@/utils/http/axios';

export function List(params) {
  return http.request({
    url: '/opsDevice/list',
    method: 'get',
    params,
  });
}

export function Delete(params) {
  return http.request({
    url: '/opsDevice/delete',
    method: 'POST',
    params,
  });
}

export function Edit(params) {
  return http.request({
    url: '/opsDevice/edit',
    method: 'POST',
    params,
  });
}

export function Status(params) {
  return http.request({
    url: '/opsDevice/status',
    method: 'POST',
    params,
  });
}

export function View(params) {
  return http.request({
    url: '/opsDevice/view',
    method: 'GET',
    params,
  });
}

export function MaxSort() {
  return http.request({
    url: '/opsDevice/maxSort',
    method: 'GET',
  });
}

export function Option() {
  return http.request({
    url: '/opsDevice/option',
    method: 'GET',
  });
}

export function CreateTerminal(params) {
  return http.request({
    url: '/opsDevice/terminal/create',
    method: 'POST',
    params,
  });
}
