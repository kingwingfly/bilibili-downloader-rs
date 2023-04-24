<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

const props = defineProps<{
    index: number,
    modelValue: {
        id: number,
        target: string,
        saveDir: string,
        working: boolean
    }[]
}>();

const emit = defineEmits<{
    (e: 'update:modelValue',
        new_value: {
            id: number,
            target: string,
            saveDir: string,
            working: boolean
        }[]
    ): void,
}>()

const target = props.modelValue[props.index].target;
const saveDir = props.modelValue[props.index].saveDir
const process = ref("");


async function switch_() {
    await invoke("switch", { id: get_id() })
    switch_working();
    init();
}

async function cancel() {
    set_working(false);
    await invoke("cancel", { id: get_id() });
}

async function re_add() {
    if (check_working()) {
        return;
    }
    set_id(await invoke("add_task", { target: target, savedir: saveDir }) as number);
    set_working(true);
    init();
}

async function rm() {
    await cancel();
    set_working(false);
    props.modelValue.splice(props.index, 1);
}

async function init() {
    while (check_working()) {
        process.value = await invoke("process", { id: get_id() });
        let [finished, total] = process.value.split("/");
        if (finished == total && total != "0") {
            set_working(false);
        }
        await new Promise(f => setTimeout(f, 10));
    }
    process.value = `Finished: ${process.value}`;
}
init()

// Some helper function
function get_info() {
    return props.modelValue[props.index]
}

function set_id(id: number) {
    get_info().id = id;
}

function get_id() {
    return get_info().id
}

function check_working() {
    return get_info().working
}

function switch_working() {
    get_info().working = !get_info().working;
}

function set_working(working: boolean) {
    get_info().working = working;
}

</script>

<template>
    <li class="task-li">
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
    <p>{{ check_working() }}</p>
</template>

<style scoped>
.task-li {
    width: auto;
    height: auto;
    background-color: #e74c3c;
}
</style>