<template>
  <n-card :segmented="{ content: true, footer: true }" footer-style="padding:10px">
    <template #header>
      通过设备浏览器信息获取浏览器指纹的插件(官方宣称其识别精度达到99.5%)
    </template>
    <div>
      指纹ID:
      <n-text type="info">
        {{ compData.murmur }}
      </n-text>
    </div>
  </n-card>
</template>
<script lang="ts" setup>
  import { reactive } from 'vue';
  import FingerprintJS from '@fingerprintjs/fingerprintjs';

  const compData = reactive({
    values: {},
    murmur: '',
  });

  const createFingerprint = async () => {
    const agent = await FingerprintJS.load();
    const result = await agent.get();
    compData.values = result.components;
    compData.murmur = result.visitorId.toUpperCase();
  };
  if (window.requestIdleCallback) {
    requestIdleCallback(() => {
      createFingerprint();
    });
  } else {
    setTimeout(() => {
      createFingerprint();
    }, 600);
  }
</script>
