<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

const props = defineProps<{
    index: number,
    modelValue: {
        id: number,
        target: string,
        saveDir: string,
        state: number
    }[]
}>();

const emit = defineEmits<{
    (e: 'update:modelValue',
        new_value: {
            id: number,
            target: string,
            saveDir: string,
            state: boolean
        }[]
    ): void,
}>()

const target = props.modelValue[props.index].target;
const saveDir = props.modelValue[props.index].saveDir
const state = ref("");
let title = ref("");


async function switch_() {
    await invoke("switch", { id: get_id() });
    await refresh_state();
    init();
}

async function cancel() {
    await invoke("cancel", { id: get_id() });
    await refresh_state();
}

async function re_add() {
    await invoke("cancel", { id: get_id() });
    set_id(await invoke("add_task", { target: target, savedir: saveDir }) as number);
    await refresh_state();
}

async function rm() {
    await cancel();
    // await new Promise(f => setTimeout(f, 1000));
    props.modelValue.splice(props.index, 1);
}

async function init() {
    let before = 0;
    while (check_state()) {
        title.value = await invoke("title", { id: get_id() }) as string;
        let c_process = await invoke("process", { id: get_id() }) as string;
        let now = parseFloat(c_process.split("Mb")[0]);
        state.value = `Working: ${c_process}; Speed: ${(now - before).toFixed(2)} Mb/s`;
        await new Promise(f => setTimeout(f, 1000));
        before = now;
        await refresh_state()
    }
    let c_state = get_info().state;
    if (c_state === 1) {
        state.value = `Pausing`;
    } else if (c_state === 2) {
        state.value = `Cancelled`;
    } else if (c_state === 3) {
        let c_process = await invoke("process", { id: get_id() });
        state.value = `Finished: ${c_process}`;
    } else {
        state.value = `Cancelled or Unknown id`;
    }
}

onMounted(() => {
    init();
})

const task_state = computed(() => ({
    'working': get_info().state === 0,
    'pausing': get_info().state === 1,
    'cancelled': get_info().state === 404,
    'finished': get_info().state === 3,
}))

watch(task_state, () => {
    if (get_info().state === 0) { init(); }
});

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

function check_state() {
    if (get_info().state === 0) {
        return true
    } else {
        return false
    }
}

async function refresh_state() {
    get_info().state = await invoke("state", { id: get_id() }) as number;
}

</script>

<template>
    <li class="task" :class="task_state">
        <div class="controller">
            <button type="button" @click="switch_()">switch state</button>
            <button type="button" @click="cancel()">cancel</button>
            <button type="button" @click="re_add()">re-add</button>
            <button type="button" @click="rm()">remove</button>
        </div>
        <div class="taskinfo">
            <p>{{ get_id() }}: {{ title }}</p>
            <p>{{ state }}</p>
        </div>
    </li>
</template>

<style scoped>
.task.working {
    background-color: #f1c40f;
    list-style: none;
    border-radius: 20px;
    padding: 10px 0px 0px 0px;
    animation: fade-in 1s forwards cubic-bezier(.64, 1.87, .64, .64);
}

@keyframes fade-in {
    0% {
        opacity: 0;
        transform: translateX(-30%);
    }

    100% {
        opacity: 1;
    }
}

.task.pausing {
    background-color: #c0392b;
    list-style: none;
    border-radius: 20px;
    padding: 10px 0px 0px 0px;
}

.task.cancelled {
    background-color: #95a5a6;
    list-style: none;
    border-radius: 20px;
    padding: 10px 0px 0px 0px;
    animation: cancel-ani 1s cubic-bezier(0.19, 1, 0.22, 1) forwards;
}

@keyframes cancel-ani {
    0% {
        opacity: 1;
    }

    50% {
        opacity: 0.5;
    }

    100% {
        opacity: 1;
    }
}

.task.finished {
    background-color: #2ecc71;
    list-style: none;
    border-radius: 20px;
    padding: 10px 0px 0px 0px;
    animation: finished-ani 1s cubic-bezier(.64, .64, .64, 1.87);
}

@keyframes finished-ani {
    0% {
        opacity: 1;
    }

    20% {
        opacity: 0.5;
        transform: translateX(10%);
    }

    100% {
        opacity: 1;
    }
}
</style>