<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import TaskCard from "../components/TaskCard.vue"

const infos = ref<{
  id: number,
  target: string,
  state: number
}[]>([]);
// c for current
const c_id = ref<number>(0);
const target = ref("");

async function addTask() {
  c_id.value = await invoke("add_task", { target: target.value }) as number;
  infos.value.push({
    id: c_id.value,
    target: target.value,
    state: 0
  });
  target.value = "";
}

async function switchAll() {
  await invoke("switch_all")
  for (let info of infos.value) {
    info.state = await invoke("state", { id: info.id }) as number;
  }
}

async function terminate() {
  await invoke("terminate");
  for (let info of infos.value) {
    info.state = await invoke("state", { id: info.id }) as number;
  }
}

</script>

<template>
  <div class="container">
    <h1>Bilibili Downloader</h1>
    <div class="inputs">
      <input id="target-input" v-model="target" placeholder="Enter a target..." />
    </div>
    <div class="btns">
      <button type="button" @click="addTask()">addTask</button>
      <button type="button" @click="switchAll()">switchAll</button>
      <button type="button" @click="terminate()">terminate</button>
    </div>
    <div class="task-list">
      <h1>Task List</h1>
      <ul>
        <TaskCard v-for="(info, index) in infos" :key="info.id" v-bind="info" v-model="infos" :index="index" />
      </ul>
    </div>
  </div>
</template>

<style scoped>
.inputs {
  width: auto;
  height: auto;
  background-color: #27ae60;
}

h1 {
  margin: 20px 0px 20px 0px;
}

button,
input {
  margin: 10px 4px 10px 4px;
}

.btns {
  width: auto;
  height: auto;
  background-color: #2980b9;
}

.task-list {
  width: auto;
  height: flex;
  background-color: #e74c3c;
  list-style: none;
}

ul {
  margin: 0;
  padding: 0px 15px 0px 15px;
}
</style>
