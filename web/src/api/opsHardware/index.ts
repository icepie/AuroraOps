import { http, jumpExport } from '@/utils/http/axios';

export function Overview(params) {
  return http.request({
    url: '/opsHardware/overview',
    method: 'GET',
    params,
  });
}

export function Export(params) {
  jumpExport('/opsHardware/export', params);
}
