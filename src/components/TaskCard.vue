<script setup lang="ts">
import { ref, Ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

const props = defineProps<{
    id: number,
    target: String,
    saveDir: String
}>();

const emit = defineEmits<{
    (e: 'rm-card'): void
}>()


const id = ref(props.id);
const process = ref("");
let working = true;


async function switch_() {
    await invoke("switch", { id: props.id })
    working = !working;
    while (working) {
        process.value = await invoke("process", { id: props.id });
    }
}

async function cancel() {
    working = false
    await invoke("cancel", { id: props.id });
}

async function re_add() {
    if (working) {
        return;
    }
    working = true
    id.value = await invoke("add_task", { target: props.target, savedir: props.saveDir }) as number;
}

async function rm() {
    await cancel();
    emit('rm-card');
}

async function init() {
    while (working) {
        process.value = await invoke("process", { id: props.id });
    }
}



init()
</script>

<template>
    <li>
        <div class="controller">
            <button type="button" @click="switch_()">switch state</button>
            <button type="button" @click="cancel()">cancel</button>
            <button type="button" @click="re_add()">re-add</button>
            <button type="button" @click="rm()">remove</button>
        </div>
        <div class="taskinfo">
            <p>{{ props.id }}</p>
            <p>{{ process }}</p>
        </div>
    </li>
</template>