import { h } from 'vue';
import { NTooltip } from 'naive-ui';

function normalizeValue(value: unknown) {
  if (value === null || value === undefined || value === '') {
    return '-';
  }
  return String(value);
}

function splitItems(value: unknown) {
  return normalizeValue(value)
    .split(/[\n;]+/)
    .map((item) => item.trim())
    .filter((item) => item && item !== '-');
}

function renderClampCell(value: unknown, className = '') {
  const text = normalizeValue(value);
  const cell = h(
    'div',
    {
      class: ['hardware-cell', className],
      title: text,
    },
    text
  );

  if (text === '-') {
    return cell;
  }

  return h(
    NTooltip,
    {
      trigger: 'hover',
      placement: 'top-start',
      style: {
        maxWidth: '720px',
        whiteSpace: 'pre-wrap',
        wordBreak: 'break-word',
      },
    },
    {
      trigger: () => cell,
      default: () => text,
    }
  );
}

function renderListCell(value: unknown, maxItems = 2) {
  const text = normalizeValue(value);
  const items = splitItems(value);

  if (!items.length) {
    return renderClampCell(text);
  }

  const visibleItems = items.slice(0, maxItems);
  const hiddenCount = Math.max(0, items.length - visibleItems.length);
  const cell = h(
    'div',
    {
      class: 'hardware-cell hardware-cell--list',
      title: text,
    },
    [
      ...visibleItems.map((item) =>
        h(
          'span',
          {
            class: 'hardware-cell__item',
          },
          item
        )
      ),
      hiddenCount
        ? h(
            'span',
            {
              class: 'hardware-cell__more',
            },
            `等 ${hiddenCount} 项`
          )
        : null,
    ]
  );

  return h(
    NTooltip,
    {
      trigger: 'hover',
      placement: 'top-start',
      style: {
        maxWidth: '760px',
        whiteSpace: 'pre-wrap',
        wordBreak: 'break-word',
      },
    },
    {
      trigger: () => cell,
      default: () => text,
    }
  );
}

export const columns = [
  {
    title: '机器名',
    key: 'deviceName',
    align: 'left',
    width: 170,
    ellipsis: { tooltip: true },
  },
  {
    title: '分组',
    key: 'groupName',
    align: 'left',
    width: 120,
    ellipsis: { tooltip: true },
  },
  {
    title: '主板',
    key: 'motherboard',
    align: 'left',
    width: 190,
    render: (row) => renderClampCell(row.motherboard),
  },
  {
    title: 'BIOS版本',
    key: 'biosVersion',
    align: 'left',
    width: 170,
    render: (row) => renderClampCell(row.biosVersion),
  },
  {
    title: 'CPU',
    key: 'cpu',
    align: 'left',
    width: 190,
    render: (row) => renderClampCell(row.cpu),
  },
  {
    title: '内存',
    key: 'memory',
    align: 'left',
    width: 210,
    render: (row) => renderListCell(row.memory, 2),
  },
  {
    title: '磁盘',
    key: 'disk',
    align: 'left',
    width: 210,
    render: (row) => renderListCell(row.disk, 2),
  },
  {
    title: '显卡',
    key: 'gpu',
    align: 'left',
    width: 180,
    render: (row) => renderListCell(row.gpu, 2),
  },
  {
    title: '网卡',
    key: 'nic',
    align: 'left',
    width: 210,
    render: (row) => renderListCell(row.nic, 3),
  },
  {
    title: '变更时间',
    key: 'changedAt',
    align: 'left',
    width: 180,
  },
];
