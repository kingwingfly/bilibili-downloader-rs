<script setup lang="ts">
// This starter template is using Vue 3 <script setup> SFCs
// Check out https://vuejs.org/api/sfc-script-setup.html#script-setup
import Downloader from "./views/Downloader.vue";
import Config from "./views/Config.vue";
import { ref } from "vue";

const hide = ref({
  hide: true
});

async function switch_show() {
  hide.value.hide = !hide.value.hide;
}
</script>

<template>
  <button type="button" @click="switch_show()">settings</button>
  <div class="side-bar" :class="hide">
    <Config />
  </div>
  <div class="main">
    <Downloader />
  </div>
</template>

<style scoped>
.main {
  background-color: #1abc9c;
}

.side-bar.hide {
  height: 0px;
  opacity: 0;
  display: none;
}
</style>




