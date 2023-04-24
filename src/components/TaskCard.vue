<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

const props = defineProps<{
    // id: number,
    // target: String,
    // saveDir: String
    index: number,
    modelValue: {
        id: number,
        target: string,
        saveDir: string
    }[]
}>();

const emit = defineEmits<{
    (e: 'rm-card'): void,
    (e: 'update:modelValue',
        new_value: {
            id: number,
            target: string,
            saveDir: string
        }[]
    ): void,
}>()

const id = props.modelValue[props.index].id;
const target = props.modelValue[props.index].target;
const saveDir = props.modelValue[props.index].saveDir
const process = ref("");
let working = true;


async function switch_() {
    await invoke("switch", { id: id })
    working = !working;
    init();
}

async function cancel() {
    working = false
    await invoke("cancel", { id: id });
}

async function re_add() {
    if (working) {
        return;
    }
    set_id(await invoke("add_task", { target: target, savedir: saveDir }) as number);
    working = true;
    init();
}

async function rm() {
    await cancel();
    working = false;
    props.modelValue.splice(props.index, 1);
}

async function init() {
    while (working) {
        process.value = await invoke("process", { id: id });
        let [finished, total] = process.value.split("/");
        if (finished == total && total != "0") {
            working = false;
        }
        await new Promise(f => setTimeout(f, 1000));
    }
    process.value = `Finished: ${process.value}`;
}

// Some helper function
function got_info() {
    return props.modelValue[props.index]
}

function set_id(id: number) {
    got_info().id = id;
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
            <p>{{ props.modelValue[props.index].id }}</p>
            <p>{{ process }}</p>
        </div>
    </li>
</template>