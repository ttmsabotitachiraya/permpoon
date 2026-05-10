<script setup lang="ts">
import { RouterView } from "vue-router";
</script>

<template>
    <div class="settings-layout">
        <div class="settings-content">
            <RouterView />
        </div>
    </div>
</template>

<style scoped>
.settings-layout {
    height: 100%;
}
.settings-content {
    height: 100%;
    overflow-y: auto;
    padding: 24px;
}
</style>
