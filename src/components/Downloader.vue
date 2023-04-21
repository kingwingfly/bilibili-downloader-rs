<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

const task_id = ref();
const target = ref("https://www.bilibili.com/video/BV1Ao4y1b7fj/?");
const save_dir = ref("../tests/video_downloads");
const state = ref("");
let working = false;


async function download() {
  task_id.value = await invoke("download", { target: target.value, savedir: save_dir.value });
  working = true;
  while (working) {
    state.value = await invoke("state", { id: task_id.value });
  }
}

async function cancel() {
  working = false
  await invoke("cancel", { id: task_id.value });
}


</script>

<template>
  <div class="card">
    <input id="target-input" v-model="target" placeholder="Enter a target..." />
    <input id="path-input" v-model="save_dir" placeholder="Enter a save_dir..." />
    <button type="button" @click="download()">download</button>
    <button type="button" @click="cancel()">cancel</button>
  </div>

  <p>{{ task_id }}</p>
  <p>{{ state }}</p>
</template>
