<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

const props = defineProps<{
    id: Number,
    // index: Number
    target: String,
    save_dir: String
}>();

const emit = defineEmits<{
    (e: 'rm-card'): void
}>()

// const emit = defineEmits(['rm-card'])

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
            <button type="button" @click="rm()">remove</button>
        </div>
        <div class="taskinfo">
            <p>{{ props.id }}</p>
            <p>{{ process }}</p>
        </div>
    </li>
</template>