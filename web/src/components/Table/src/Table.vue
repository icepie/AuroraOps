<template>
  <div ref="wrapRef" class="basic-table" :class="{ 'basic-table--full-height': fullHeight }">
    <div class="table-toolbar">
      <!--顶部左侧区域-->
      <div class="flex items-center table-toolbar-left">
        <template v-if="title">
          <div class="table-toolbar-left-title">
            {{ title }}
            <n-tooltip trigger="hover" v-if="titleTooltip">
              <template #trigger>
                <n-icon size="18" class="ml-1 text-gray-400 cursor-pointer">
                  <QuestionCircleOutlined />
                </n-icon>
              </template>
              {{ titleTooltip }}
            </n-tooltip>
          </div>
        </template>
        <slot name="tableTitle"></slot>
      </div>

      <div class="flex items-center table-toolbar-right" v-show="showTopRight">
        <!--顶部右侧区域-->
        <slot name="toolbar"></slot>

        <!--斑马纹-->
        <n-tooltip trigger="hover">
          <template #trigger>
            <div class="table-toolbar-right-icon table-toolbar-right-icon--switch">
              <n-switch v-model:value="isStriped" @update:value="setStriped" />
            </div>
          </template>
          <span>表格斑马纹</span>
        </n-tooltip>
        <n-divider vertical />

        <!--刷新-->
        <n-tooltip trigger="hover">
          <template #trigger>
            <div class="table-toolbar-right-icon" @click="reload">
              <n-icon size="18">
                <ReloadOutlined />
              </n-icon>
            </div>
          </template>
          <span>刷新</span>
        </n-tooltip>

        <!--密度-->
        <n-tooltip trigger="hover">
          <template #trigger>
            <div class="table-toolbar-right-icon">
              <n-dropdown
                @select="densitySelect"
                trigger="click"
                :options="densityOptions"
                v-model:value="tableSize"
              >
                <n-icon size="18">
                  <ColumnHeightOutlined />
                </n-icon>
              </n-dropdown>
            </div>
          </template>
          <span>密度</span>
        </n-tooltip>

        <!--表格设置单独抽离成组件-->
        <ColumnSetting :openChecked="openChecked" />
      </div>
    </div>
    <div ref="tableWrapRef" class="s-table">
      <n-data-table
        ref="tableElRef"
        v-bind="getBindValues"
        :striped="isStriped"
        :pagination="pagination"
        @update:page="updatePage"
        @update:page-size="updatePageSize"
        @update:checked-row-keys="updateCheckedRowKeys"
        @update:expanded-row-keys="updateExpandedRowKeys"
      >
        <template #[item]="data" v-for="item in Object.keys($slots)" :key="item">
          <slot :name="item" v-bind="data"></slot>
        </template>
      </n-data-table>
    </div>
  </div>
</template>

<script lang="ts">
  import {
    ref,
    defineComponent,
    reactive,
    unref,
    toRaw,
    computed,
    toRefs,
    onMounted,
    watch,
    nextTick,
    onBeforeUnmount,
  } from 'vue';
  import { ReloadOutlined, ColumnHeightOutlined, QuestionCircleOutlined } from '@vicons/antd';
  import { createTableContext } from './hooks/useTableContext';

  import ColumnSetting from './components/settings/ColumnSetting.vue';

  import { useLoading } from './hooks/useLoading';
  import { useColumns } from './hooks/useColumns';
  import { useDataSource } from './hooks/useDataSource';
  import { usePagination } from './hooks/usePagination';

  import { basicProps } from './props';

  import { BasicTableProps } from './types/table';

  import { useWindowSizeFn } from '@/hooks/event/useWindowSizeFn';

  const densityOptions = [
    {
      type: 'menu',
      label: '紧凑',
      key: 'small',
    },
    {
      type: 'menu',
      label: '默认',
      key: 'medium',
    },
    {
      type: 'menu',
      label: '宽松',
      key: 'large',
    },
  ];

  export default defineComponent({
    components: {
      ReloadOutlined,
      ColumnHeightOutlined,
      ColumnSetting,
      QuestionCircleOutlined,
    },
    props: {
      ...basicProps,
    },
    emits: [
      'fetch-success',
      'fetch-error',
      'update:checked-row-keys',
      'update:expanded-row-keys',
      'edit-end',
      'edit-cancel',
      'edit-row-end',
      'edit-change',
    ],
    setup(props, { emit }) {
      const deviceHeight = ref(150);
      const tableElRef = ref<ComponentRef>(null);
      const wrapRef = ref<Nullable<HTMLDivElement>>(null);
      const tableWrapRef = ref<Nullable<HTMLDivElement>>(null);
      const tableWrapWidth = ref(0);
      let resizeObserver: ResizeObserver | null = null;
      let resizeRaf = 0;
      const isStriped = ref(false);
      const tableData = ref<Recordable[]>([]);
      const innerPropsRef = ref<Partial<BasicTableProps>>();

      const getProps = computed(() => {
        return { ...props, ...unref(innerPropsRef) } as BasicTableProps;
      });

      const { getLoading, setLoading } = useLoading(getProps);

      const { getPaginationInfo, setPagination } = usePagination(getProps);

      const { getDataSourceRef, getDataSource, setTableData, getRowKey, reload } = useDataSource(
        getProps,
        {
          getPaginationInfo,
          setPagination,
          tableData,
          setLoading,
        },
        emit
      );

      const { getPageColumns, setColumns, getColumns, getCacheColumns, setCacheColumnsField } =
        useColumns(getProps);

      const state = reactive({
        tableSize: unref(getProps as any).size || 'small',
        isColumnSetting: false,
      });

      //页码切换
      function updatePage(page) {
        setPagination({ page: page });
        reload();
      }

      //分页数量切换
      function updatePageSize(size) {
        setPagination({ page: 1, pageSize: size });
        reload();
      }

      //密度切换
      function densitySelect(e) {
        state.tableSize = e;
      }

      //选中行
      function updateCheckedRowKeys(rowKeys) {
        emit('update:checked-row-keys', rowKeys);
      }

      function updateExpandedRowKeys(rowKeys) {
        emit('update:expanded-row-keys', rowKeys);
      }

      //获取表格大小
      const getTableSize = computed(() => state.tableSize);

      //组装表格信息
      const getBindValues = computed(() => {
        const bindProps = unref(getProps);
        const tableData = unref(getDataSourceRef);
        const scrollX = resolveScrollX((bindProps as any).scrollX);
        const maxHeight =
          unref(getCanResize) && unref(getFullHeight) && unref(deviceHeight) > 0
            ? unref(deviceHeight)
            : undefined;
        return {
          ...bindProps,
          loading: unref(getLoading),
          columns: toRaw(unref(getPageColumns)),
          rowKey: unref(getRowKey),
          data: tableData,
          size: unref(getTableSize),
          remote: true,
          ...(scrollX !== undefined ? { scrollX } : {}),
          ...(maxHeight ? { maxHeight } : {}),
          style: (bindProps as any).style,
        };
      });

      //获取分页信息
      const pagination = computed(() => toRaw(unref(getPaginationInfo)));

      function setProps(props: Partial<BasicTableProps>) {
        innerPropsRef.value = { ...unref(innerPropsRef), ...props };
      }

      const setStriped = (value: boolean) => (isStriped.value = value);

      const tableAction = {
        reload,
        setColumns,
        setLoading,
        setProps,
        getColumns,
        getDataSource,
        setTableData,
        getPageColumns,
        getCacheColumns,
        setCacheColumnsField,
        emit,
      };

      const getCanResize = computed(() => {
        const { canResize } = unref(getProps);
        return canResize;
      });

      const getFullHeight = computed(() => {
        const { fullHeight } = unref(getProps);
        return fullHeight;
      });

      function resolveScrollX(scrollX: unknown) {
        const wrapWidth = unref(tableWrapWidth);
        if (typeof scrollX === 'number') {
          return Math.max(scrollX, wrapWidth);
        }
        if (typeof scrollX === 'string' && scrollX.trim() !== '') {
          const parsed = Number(scrollX);
          if (Number.isFinite(parsed)) {
            return Math.max(parsed, wrapWidth);
          }
        }
        return scrollX;
      }

      function scheduleTableHeightCompute() {
        if (resizeRaf) {
          window.cancelAnimationFrame(resizeRaf);
        }
        resizeRaf = window.requestAnimationFrame(() => {
          resizeRaf = 0;
          computeTableHeight();
        });
      }

      function computeTableHeight() {
        if (!unref(getCanResize)) return;
        const tableWrapEl = unref(tableWrapRef);
        const table = unref(tableElRef);
        const tableEl: any = table?.$el;
        const measureEl = tableWrapEl || tableEl;
        if (!measureEl?.getBoundingClientRect) return;

        const viewportHeight = window.document.documentElement.clientHeight || window.innerHeight;
        const measuredHeight = tableWrapEl?.clientHeight || 0;
        tableWrapWidth.value = tableWrapEl ? Math.ceil(tableWrapEl.clientWidth) : 0;
        const { top } = measureEl.getBoundingClientRect();
        const viewportHeightFallback = viewportHeight - top - 12;
        const resizeHeightOffset = props.resizeHeightOffset || 0;
        const legacyAutoHeight = resizeHeightOffset <= -1000;
        if (!unref(getFullHeight) && legacyAutoHeight) {
          deviceHeight.value = 0;
          return;
        }
        const availableHeight = measuredHeight > 0 ? measuredHeight : viewportHeightFallback;
        if (availableHeight <= 0) return;
        let height = availableHeight - (legacyAutoHeight ? 0 : resizeHeightOffset);
        const maxHeight = props.maxHeight;
        height = maxHeight && maxHeight < height ? maxHeight : height;
        deviceHeight.value = Math.max(180, Math.floor(height));
      }

      useWindowSizeFn(scheduleTableHeightCompute, 120);

      onMounted(() => {
        nextTick(() => {
          resizeObserver = new ResizeObserver(scheduleTableHeightCompute);
          const wrapEl = unref(wrapRef);
          const tableWrapEl = unref(tableWrapRef);
          wrapEl && resizeObserver.observe(wrapEl);
          tableWrapEl && resizeObserver.observe(tableWrapEl);
          scheduleTableHeightCompute();
        });
      });

      onBeforeUnmount(() => {
        resizeObserver?.disconnect();
        resizeObserver = null;
        if (resizeRaf) {
          window.cancelAnimationFrame(resizeRaf);
          resizeRaf = 0;
        }
      });

      watch(
        [() => props.columns, () => props.pagination, () => state.tableSize],
        async () => {
          await nextTick();
          scheduleTableHeightCompute();
        },
        { deep: true }
      );

      watch(getDataSourceRef, async () => {
        await nextTick();
        scheduleTableHeightCompute();
      });

      createTableContext({ ...tableAction, wrapRef, getBindValues });

      return {
        ...toRefs(state),
        wrapRef,
        tableWrapRef,
        tableElRef,
        getBindValues,
        getDataSource,
        setTableData,
        densityOptions,
        reload,
        updateCheckedRowKeys,
        updateExpandedRowKeys,
        densitySelect,
        updatePage,
        updatePageSize,
        pagination,
        tableAction,
        setStriped,
        isStriped,
      };
    },
  });
</script>
<style lang="less" scoped>
  .basic-table {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: auto;
    min-width: 0;
    min-height: 0;
  }

  .basic-table--full-height {
    flex: 1 1 auto;
    height: 100%;
  }

  .table-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    min-width: 0;
    min-height: 28px;
    padding: 0 0 6px 0;
    flex-wrap: wrap;

    &-left {
      display: flex;
      align-items: center;
      justify-content: flex-start;
      flex: 1 1 360px;
      min-width: 0;
      gap: 6px;
      flex-wrap: wrap;

      &-title {
        display: flex;
        align-items: center;
        justify-content: flex-start;
        min-width: 0;
        font-size: 14px;
        font-weight: 600;
      }
    }

    &-right {
      display: flex;
      align-items: center;
      justify-content: flex-end;
      flex: 0 0 auto;
      min-width: 0;
      gap: 6px;
      flex-wrap: nowrap;
      white-space: nowrap;

      &-icon {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 24px;
        min-width: 24px;
        height: 24px;
        margin-left: 0;
        border-radius: 4px;
        font-size: 14px;
        cursor: pointer;
        color: var(--text-color);

        &:hover {
          color: #1890ff;
        }
      }

      &-icon--switch {
        width: auto;
        min-width: 36px;
      }
    }
  }

  .s-table {
    display: flex;
    flex: 1 1 auto;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
  }

  .s-table :deep(.n-data-table) {
    flex: 1 1 auto;
    min-width: 0;
    min-height: 0;
    max-height: 100%;
  }

  .s-table :deep(.n-data-table-wrapper) {
    flex: 0 1 auto;
    min-width: 0;
    min-height: 0;
  }

  .s-table :deep(.n-data-table-base-table) {
    display: flex;
    flex: 0 1 auto;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
  }

  .s-table :deep(.n-data-table-base-table-body) {
    min-height: 0;
  }

  .s-table :deep(.n-data-table__pagination) {
    flex: 0 0 auto;
    margin: 8px 0 0;
    justify-content: flex-end;
    row-gap: 6px;
    background: var(--n-color);
  }

  @media (max-width: 768px) {
    .table-toolbar {
      align-items: flex-start;
      flex-direction: column;
      gap: 6px;
    }

    .table-toolbar-left,
    .table-toolbar-right {
      width: 100%;
      flex-basis: auto;
    }

    .table-toolbar-right {
      justify-content: flex-start;
      flex-wrap: wrap;
      white-space: normal;
    }

    .s-table :deep(.n-data-table__pagination) {
      justify-content: flex-start;
    }
  }

  .table-toolbar-inner-popover-title {
    padding: 2px 0;
  }
</style>
