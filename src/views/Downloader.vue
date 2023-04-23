<script setup lang="ts">
import { ref, Ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import TaskCard from "../components/TaskCard.vue"

const infos = ref<{
  id: number,
  target: string,
  saveDir: string
}[]>([]);
// c for current
const c_id = ref<number>(0);
const target = ref("https://www.bilibili.com/video/BV1Ao4y1b7fj/?");
const saveDir = ref("../tests/video_downloads");

async function download() {
  c_id.value = await invoke("add_task", { target: target.value, savedir: saveDir.value }) as number;
  infos.value.push({
    id: c_id.value,
    target: target.value,
    saveDir: saveDir.value
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
    <input id="path-input" v-model="saveDir" placeholder="Enter a saveDir..." />
    <button type="button" @click="download()">download</button>
    <button type="button" @click="switchAll()">switchAll</button>
    <button type="button" @click="terminate()">terminate</button>
    <ul>
      <task-card v-for="(info, index) in infos" :key="info.id" v-bind="info" @rm-card="rm_card(index)" />
    </ul>
  </div>
</template>
