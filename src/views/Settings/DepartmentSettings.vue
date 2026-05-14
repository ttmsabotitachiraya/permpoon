<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useDepartmentStore } from "../../stores/departmentStore";
import { useConnectionStore } from "../../stores/connectionStore";
import { Search, Trash2, Building2, AlertCircle } from "lucide-vue-next";

const deptStore = useDepartmentStore();
const connStore = useConnectionStore();

const depcodeInput = ref("");
const departmentName = ref("");
const isFetching = ref(false);
const fetchError = ref("");
const saveMsg = ref("");
const saveError = ref("");
const confirmDeleteId = ref<number | null>(null);

onMounted(async () => {
    await deptStore.loadAll();
});

async function searchDepartment() {
    if (!depcodeInput.value.trim()) return;
    if (!connStore.isConnected) {
        fetchError.value =
            "ยังไม่ได้เชื่อมต่อฐานข้อมูล กรุณาตั้งค่าที่เมนู 'การเชื่อมต่อ'";
        return;
    }
    isFetching.value = true;
    fetchError.value = "";
    departmentName.value = "";
    try {
        const result = await invoke<{
            depcode: string;
            department: string;
        } | null>("search_department_by_depcode", {
            config: connStore.config,
            depcode: depcodeInput.value.trim(),
        });
        if (result) {
            departmentName.value = result.department;
        } else {
            fetchError.value = `ไม่พบรหัสแผนก ${depcodeInput.value.trim()} ในฐานข้อมูล HOSxP`;
        }
    } catch (e: any) {
        fetchError.value = String(e);
    } finally {
        isFetching.value = false;
    }
}

async function handleSave() {
    if (!depcodeInput.value.trim() || !departmentName.value.trim()) {
        saveError.value = "กรุณาค้นหาแผนกก่อนบันทึก";
        return;
    }
    saveError.value = "";
    try {
        await deptStore.saveDept({
            depcode: depcodeInput.value.trim(),
            department: departmentName.value.trim(),
            is_enabled: true,
        });
        saveMsg.value = `บันทึกแผนก "${departmentName.value}" สำเร็จ`;
        depcodeInput.value = "";
        departmentName.value = "";
        setTimeout(() => (saveMsg.value = ""), 3000);
    } catch (e: any) {
        saveError.value = String(e);
    }
}

function requestRemove(id: number) {
    confirmDeleteId.value = id;
}

function cancelRemove() {
    confirmDeleteId.value = null;
}

async function handleRemove(id: number) {
    confirmDeleteId.value = null;
    try {
        await deptStore.removeDept(id);
    } catch (e: unknown) {
        saveError.value = `ลบไม่สำเร็จ: ${String(e)}`;
    }
}
</script>

<template>
    <div class="dept-settings">
        <div class="card mb-4">
            <div class="card-header">
                <Building2 :size="18" style="color: var(--brand-orange)" />
                เพิ่มแผนกบริการ
            </div>
            <div class="card-body">
                <div class="dept-search-row">
                    <div style="flex: 1">
                        <label class="form-label">รหัสแผนก (depcode)</label>
                        <div class="search-input-row">
                            <input
                                v-model="depcodeInput"
                                class="form-input font-mono"
                                placeholder="เช่น 007"
                                @keydown.enter="searchDepartment"
                            />
                            <button
                                class="btn btn-secondary"
                                @click="searchDepartment"
                                :disabled="isFetching"
                            >
                                <Search :size="16" />
                                {{ isFetching ? "..." : "ค้นหา" }}
                            </button>
                        </div>
                    </div>
                    <div style="flex: 2">
                        <label class="form-label">ชื่อแผนก</label>
                        <input
                            v-model="departmentName"
                            class="form-input"
                            placeholder="ชื่อแผนกจากการค้นหา"
                            :disabled="!departmentName"
                        />
                    </div>
                </div>

                <div v-if="fetchError" class="alert alert-error mt-2">
                    <AlertCircle :size="16" /> {{ fetchError }}
                </div>

                <div v-if="departmentName" class="save-section">
                    <button
                        class="btn btn-primary"
                        @click="handleSave"
                        :disabled="!departmentName"
                    >
                        บันทึก
                    </button>
                </div>

                <div v-if="saveMsg" class="alert alert-success mt-2">
                    {{ saveMsg }}
                </div>
                <div v-if="saveError" class="alert alert-error mt-2">
                    <AlertCircle :size="16" /> {{ saveError }}
                </div>
            </div>
        </div>

        <div class="card">
            <div class="card-header">
                <Building2 :size="18" style="color: var(--brand-orange)" />
                แผนกบริการที่ตั้งค่าแล้ว
                <span class="badge badge-orange" style="margin-left: auto">
                    {{ deptStore.items.length }} รายการ
                </span>
            </div>
            <div class="card-body" style="padding: 0">
                <div v-if="deptStore.items.length === 0" class="empty-state">
                    <Building2
                        :size="32"
                        style="color: var(--muted); margin-bottom: 8px"
                    />
                    <p>ยังไม่มีแผนกที่ตั้งค่า</p>
                </div>
                <table v-else class="table dept-table" style="font-size: 13px">
                    <thead>
                        <tr>
                            <th>รหัส</th>
                            <th>ชื่อแผนก</th>
                            <th>สถานะ</th>
                            <th></th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr v-for="item in deptStore.items" :key="item.id">
                            <td>
                                <span class="badge badge-gray font-mono">{{
                                    item.depcode
                                }}</span>
                            </td>
                            <td>{{ item.department }}</td>
                            <td>
                                <button
                                    class="toggle"
                                    :class="{ active: item.is_enabled }"
                                    @click="
                                        deptStore.toggleEnabled(
                                            item.id,
                                            !item.is_enabled,
                                        )
                                    "
                                ></button>
                            </td>
                            <td style="text-align: right">
                                <button
                                    class="btn btn-danger btn-sm"
                                    @click="requestRemove(item.id)"
                                >
                                    <Trash2 :size="13" />
                                </button>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>

        <div
            v-if="confirmDeleteId !== null"
            class="delete-overlay"
            @click.self="cancelRemove"
        >
            <div class="delete-dialog">
                <p class="delete-dialog-title">ยืนยันการลบ?</p>
                <p class="delete-dialog-sub">
                    ต้องการลบ "{{
                        deptStore.items.find((i) => i.id === confirmDeleteId)
                            ?.department
                    }}" ออกจากระบบ
                </p>
                <div class="delete-dialog-actions">
                    <button
                        class="btn btn-danger"
                        @click="handleRemove(confirmDeleteId!)"
                    >
                        ยืนยัน
                    </button>
                    <button class="btn btn-ghost" @click="cancelRemove">
                        ยกเลิก
                    </button>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
.dept-settings {
    display: flex;
    flex-direction: column;
    max-width: 800px;
}
.mb-4 {
    margin-bottom: 16px;
}
.mt-2 {
    margin-top: 8px;
}
.dept-search-row {
    display: flex;
    gap: 16px;
    align-items: flex-start;
}
.search-input-row {
    display: flex;
    gap: 8px;
}
.search-input-row .form-input {
    flex: 1;
}
.save-section {
    margin-top: 12px;
    display: flex;
    justify-content: flex-end;
}
.empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 40px;
    color: var(--slate);
}
.dept-table {
    table-layout: fixed;
}
.dept-table th:nth-child(1) {
    width: 120px;
}
.dept-table th:nth-child(3) {
    width: 100px;
}
.dept-table th:nth-child(4) {
    width: 80px;
}

.delete-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.35);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 300;
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
