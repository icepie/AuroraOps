<template>
  <n-modal
    v-model:show="showModal"
    preset="card"
    title="预览"
    :bordered="false"
    :style="{ width: previewWidth }"
    content-class="file-preview-modal"
  >
    <div class="file-preview-modal__body">
      <n-image preview-disabled class="file-preview-modal__image" :src="previewUrl" :on-error="errorImg" />
    </div>
  </n-modal>
</template>

<script setup lang="ts">
  import { errorImg } from '@/utils/hotgo';
  import { computed, ref } from 'vue';
  const showModal = ref(false);
  const previewUrl = ref('');
  const previewWidth = computed(() => {
    if (typeof window === 'undefined') {
      return '640px';
    }
    return `${Math.min(720, Math.max(320, window.innerWidth - 48))}px`;
  });

  function openPreview(url) {
    showModal.value = true;
    previewUrl.value = url;
  }

  defineExpose({
    openPreview,
  });
</script>

<style scoped lang="less">
  .file-preview-modal__body {
    display: flex;
    align-items: center;
    justify-content: center;
    max-height: calc(100vh - 180px);
    overflow: auto;
  }

  .file-preview-modal__image {
    display: block;
    max-width: 100%;
    max-height: calc(100vh - 180px);
  }

  .file-preview-modal__image :deep(img) {
    display: block;
    max-width: 100%;
    max-height: calc(100vh - 180px);
    width: auto;
    height: auto;
    object-fit: contain;
  }
</style>
