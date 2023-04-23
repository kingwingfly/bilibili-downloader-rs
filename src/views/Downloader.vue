<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import TaskCard from "../components/TaskCard.vue"

const infos = ref<{
  id: number,
  target: string,
  save_dir: string
}[]>([]);
// c for current
const c_id = ref<number>();
const target = ref("https://www.bilibili.com/video/BV1Ao4y1b7fj/?");
const save_dir = ref("../tests/video_downloads");

async function download() {
  c_id.value = await invoke("add_task", { target: target.value, savedir: save_dir.value }) as number;
  infos.value.push({
    id: c_id.value,
    target: target.value,
    save_dir: save_dir.value
  })
}

async function switchAll() {
  await invoke("switch_all")
}

async function terminate() {
  await invoke("terminate");
}

async function rm_card(index: number) {
  infos.value.splice(index, 1);
}
</script>

<template>
  <div class="container">
    <input id="target-input" v-model="target" placeholder="Enter a target..." />
    <input id="path-input" v-model="save_dir" placeholder="Enter a save_dir..." />
    <button type="button" @click="download()">download</button>
    <button type="button" @click="switchAll()">switchAll</button>
    <button type="button" @click="terminate()">terminate</button>
    <ul>
      <task-card v-for="(info, index) in infos" :key="info.id" v-bind="info" :index="index" @rm-card="rm_card(index)" />
    </ul>
  </div>
</template>
