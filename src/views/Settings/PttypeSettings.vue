<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { usePttypeStore } from "../../stores/pttypeStore";
import { useConnectionStore } from "../../stores/connectionStore";
import { Search, Plus, Trash2, Shield, AlertCircle } from "lucide-vue-next";
import type { PttypeRow } from "../../types/pttype";

const pttypeStore = usePttypeStore();
const connStore = useConnectionStore();

const searchCode = ref("");
const searchResults = ref<PttypeRow[]>([]);
const aliasInput = ref("");
const isSearching = ref(false);
const searchError = ref("");
const addError = ref("");
const addMsg = ref("");
const confirmDeleteId = ref<number | null>(null);

onMounted(() => pttypeStore.loadAll());

async function doSearch() {
    if (!searchCode.value.trim()) return;
    if (!connStore.isConnected) {
        searchError.value = "ยังไม่ได้เชื่อมต่อฐานข้อมูล";
        return;
    }
    isSearching.value = true;
    searchError.value = "";
    try {
        const results = await invoke<PttypeRow[]>("search_pttype_by_hipdata", {
            config: connStore.config,
            hipdataCode: searchCode.value.trim().toUpperCase(),
        });
        searchResults.value = results;
        if (results.length > 0) {
            aliasInput.value = searchCode.value.trim().toUpperCase();
        }
    } catch (e: any) {
        searchError.value = String(e);
    } finally {
        isSearching.value = false;
    }
}

async function doAdd() {
    if (!aliasInput.value.trim() || !searchCode.value.trim()) return;
    addError.value = "";
    try {
        await invoke("add_pttype_group", {
            alias: aliasInput.value.trim().toUpperCase(),
            hipdataCode: searchCode.value.trim().toUpperCase(),
        });
        addMsg.value = `เพิ่ม "${aliasInput.value}" สำเร็จ`;
        await pttypeStore.loadAll();
        setTimeout(() => (addMsg.value = ""), 3000);
    } catch (e: any) {
        addError.value = String(e);
    }
}

function requestDelete(id: number) {
    confirmDeleteId.value = id;
}

function cancelDelete() {
    confirmDeleteId.value = null;
}

async function doRemove(id: number) {
    confirmDeleteId.value = null;
    try {
        await pttypeStore.removeGroup(id);
    } catch (e: unknown) {
        addError.value = `ลบไม่สำเร็จ: ${String(e)}`;
    }
}
</script>

<template>
    <div class="pttype-settings">
        <!-- Search -->
        <div class="card mb-4">
            <div class="card-header">
                <Search :size="18" style="color: var(--brand-orange)" />
                ค้นหา hipdata_code จาก HOSxP
            </div>
            <div class="card-body">
                <div class="search-row">
                    <div style="flex: 1">
                        <label class="form-label"
                            >hipdata_code (เช่น UCS, OFC, WEL)</label
                        >
                        <input
                            v-model="searchCode"
                            class="form-input"
                            placeholder="UCS"
                            @keydown.enter="doSearch"
                            style="text-transform: uppercase"
                        />
                    </div>
                    <div style="align-self: flex-end">
                        <button
                            class="btn btn-secondary"
                            @click="doSearch"
                            :disabled="isSearching"
                        >
                            <Search :size="16" />
                            {{ isSearching ? "กำลังค้นหา..." : "ค้นหา" }}
                        </button>
                    </div>
                </div>
                <div v-if="searchError" class="alert alert-error mt-2">
                    <AlertCircle :size="16" />{{ searchError }}
                </div>

                <!-- Results -->
                <div
                    v-if="searchResults.length > 0"
                    class="search-results mt-4"
                >
                    <div class="results-label">
                        พบ {{ searchResults.length }} รายการ (hipdata_code = "{{
                            searchCode.toUpperCase()
                        }}")
                    </div>
                    <div class="pttype-list">
                        <div
                            v-for="r in searchResults"
                            :key="r.pttype"
                            class="pttype-item"
                        >
                            <span class="badge badge-gray font-mono">{{
                                r.pttype
                            }}</span>
                            <span>{{ r.name }}</span>
                        </div>
                    </div>
                </div>
                <div
                    v-else-if="
                        !isSearching && searchCode && searchResults.length === 0
                    "
                    class="alert alert-warning mt-2"
                >
                    <AlertCircle :size="16" />
                    ไม่พบ hipdata_code นี้ในฐานข้อมูล
                </div>

                <!-- Add -->
                <div v-if="searchResults.length > 0" class="add-row mt-4">
                    <div style="flex: 1">
                        <label class="form-label"
                            >ชื่อย่อ (alias) สำหรับแสดงในโปรแกรม</label
                        >
                        <input
                            v-model="aliasInput"
                            class="form-input"
                            placeholder="UCS"
                            style="text-transform: uppercase"
                        />
                    </div>
                    <div style="align-self: flex-end">
                        <button class="btn btn-primary" @click="doAdd">
                            <Plus :size="16" />
                            เพิ่มรายการ
                        </button>
                    </div>
                </div>
                <div v-if="addMsg" class="alert alert-success mt-2">
                    {{ addMsg }}
                </div>
                <div v-if="addError" class="alert alert-error mt-2">
                    <AlertCircle :size="16" />{{ addError }}
                </div>
            </div>
        </div>

        <!-- List -->
        <div class="card">
            <div class="card-header">
                <Shield :size="18" style="color: var(--brand-orange)" />
                สิทธิ์ที่ตั้งค่าแล้ว
                <span class="badge badge-orange" style="margin-left: auto"
                    >{{ pttypeStore.groups.length }} รายการ</span
                >
            </div>
            <div class="card-body" style="padding: 0">
                <div v-if="pttypeStore.groups.length === 0" class="empty-state">
                    <Shield
                        :size="32"
                        style="color: var(--muted); margin-bottom: 8px"
                    />
                    <p>ยังไม่มีสิทธิ์ที่ตั้งค่า</p>
                </div>
                <table v-else class="table">
                    <thead>
                        <tr>
                            <th>Alias</th>
                            <th>hipdata_code</th>
                            <th>icode ที่ใช้</th>
                            <th></th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr v-for="g in pttypeStore.groups" :key="g.id">
                            <td>
                                <span class="alias-plain">{{ g.alias }}</span>
                            </td>
                            <td>
                                <span class="font-mono text-sm">{{
                                    g.hipdata_code
                                }}</span>
                            </td>
                            <td>
                                <span class="badge badge-gray"
                                    >{{ g.icode_count ?? 0 }} รายการ</span
                                >
                            </td>
                            <td style="text-align: right">
                                <button
                                    class="btn btn-danger btn-sm"
                                    @click="requestDelete(g.id)"
                                >
                                    <Trash2 :size="14" />
                                </button>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>

        <!-- Delete Confirmation Popup -->
        <div
            v-if="confirmDeleteId !== null"
            class="delete-overlay"
            @click.self="cancelDelete"
        >
            <div class="delete-dialog">
                <p class="delete-dialog-title">ยืนยันการลบ?</p>
                <p class="delete-dialog-sub">
                    ต้องการลบสิทธิ์ "{{
                        pttypeStore.groups.find((g) => g.id === confirmDeleteId)
                            ?.alias
                    }}" ออกจากระบบ
                </p>
                <div class="delete-dialog-actions">
                    <button
                        class="btn btn-danger"
                        @click="doRemove(confirmDeleteId!)"
                    >
                        ยืนยัน
                    </button>
                    <button class="btn btn-ghost" @click="cancelDelete">
                        ยกเลิก
                    </button>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
.pttype-settings {
    display: flex;
    flex-direction: column;
    gap: 0;
    max-width: 700px;
}
.mb-4 {
    margin-bottom: 16px;
}
.mt-2 {
    margin-top: 8px;
}
.mt-4 {
    margin-top: 16px;
}
.search-row,
.add-row {
    display: flex;
    gap: 12px;
    align-items: flex-start;
}
.results-label {
    font-size: 13px;
    font-weight: 500;
    color: var(--slate);
    margin-bottom: 8px;
}
.pttype-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
}
.pttype-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 0;
    font-size: 14px;
}
.empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 40px;
    color: var(--slate);
}
.alias-plain {
    font-size: 14px;
    color: var(--charcoal);
    font-weight: 500;
}
.delete-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.35);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 200;
}
.delete-dialog {
    background: white;
    border-radius: var(--radius-lg);
    padding: 24px 28px;
    min-width: 300px;
    box-shadow: var(--shadow-lg);
    display: flex;
    flex-direction: column;
    gap: 8px;
}
.delete-dialog-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--ink);
}
.delete-dialog-sub {
    font-size: 13px;
    color: var(--slate);
    margin-bottom: 8px;
}
.delete-dialog-actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
}
</style>
