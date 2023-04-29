<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

const cookie = ref("");
const saveDir = ref("");
const parts = ref<number>();
const ffmpeg = ref("");
const message = ref("");

async function init() {
    [cookie.value, saveDir.value, parts.value, ffmpeg.value] = await invoke("read_config") as [string, string, number, string];
}

async function submit() {
    await invoke("submit_config", { cookie: cookie.value, savedir: saveDir.value, parts: parts.value, ffmpeg: ffmpeg.value });
    message.value = "Configuration successful, please restart the app to apply the modification";
}

onMounted(() => {
    init()
})
</script>

<template>
    <div class="config-container">
        <div class="config">
            <label for="cookie-input">Cookie:</label>
            <input id="cookie-input" v-model="cookie" placeholder="Enter your cookie..." />
            <label for="path-input">SaveDir:</label>
            <input id="path-input" v-model="saveDir" placeholder="Enter a saveDir..." />
            <label for="parts-input">ThreadNumber:</label>
            <input id="parts-input" v-model="parts" placeholder="Enter the thread number..." />
            <label for="ffmpeg-input">ffmpeg path:</label>
            <input id="ffmpeg-input" v-model="ffmpeg" placeholder="Enter your ffmpeg path..." />
        </div>
        <div class="btns">
            <button type="button" @click="submit()">submit</button>
        </div>
        <div class="message">{{ message }}</div>
    </div>
</template>

<style scoped>
.config-container {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
}

.config {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    gap: 10px;
    width: auto;
}

.btns {
    display: flex;
    justify-content: center;
    align-items: center;
    margin: 6px;
}
</style>
