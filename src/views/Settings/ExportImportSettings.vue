<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import { useIcodeStore } from "../../stores/icodeStore";
import { usePttypeStore } from "../../stores/pttypeStore";
import { useDepartmentStore } from "../../stores/departmentStore";
import {
    Download,
    Upload,
    FileJson,
    CheckSquare,
    Square,
    AlertCircle,
    Check,
} from "lucide-vue-next";

interface ExportOptions {
    include_pttype_groups: boolean;
    pttype_group_ids: number[] | null;
    include_icode: boolean;
    icode_ids: number[] | null;
    include_departments: boolean;
    department_ids: number[] | null;
}

interface ImportSummary {
    pttype_groups_added: number;
    pttype_groups_skipped: number;
    icode_configs_added: number;
    icode_configs_updated: number;
    department_configs_added: number;
    department_configs_updated: number;
}

const icodeStore = useIcodeStore();
const pttypeStore = usePttypeStore();
const deptStore = useDepartmentStore();

// ── Export state ─────────────────────────────────────────────────────────────
const includePttype = ref(true);
const pttypeMode = ref<"all" | "select">("all");
const selectedPttypeIds = ref<number[]>([]);
const includeIcode = ref(true);
const icodeMode = ref<"all" | "select">("all");
const selectedIcodeIds = ref<number[]>([]);
const includeDepartments = ref(true);
const departmentMode = ref<"all" | "select">("all");
const selectedDepartmentIds = ref<number[]>([]);
const isExporting = ref(false);
const exportError = ref("");

const allPttypesSelected = computed(
    () =>
        pttypeStore.groups.length > 0 &&
        selectedPttypeIds.value.length === pttypeStore.groups.length,
);

const allIcodesSelected = computed(
    () =>
        icodeStore.items.length > 0 &&
        selectedIcodeIds.value.length === icodeStore.items.length,
);

const allDepartmentsSelected = computed(
    () =>
        deptStore.items.length > 0 &&
        selectedDepartmentIds.value.length === deptStore.items.length,
);

function toggleSelectAllPttypes() {
    if (allPttypesSelected.value) selectedPttypeIds.value = [];
    else selectedPttypeIds.value = pttypeStore.groups.map((g) => g.id);
}

function togglePttype(id: number) {
    const idx = selectedPttypeIds.value.indexOf(id);
    if (idx >= 0) selectedPttypeIds.value.splice(idx, 1);
    else selectedPttypeIds.value.push(id);
}

function toggleSelectAllIcodes() {
    if (allIcodesSelected.value) selectedIcodeIds.value = [];
    else selectedIcodeIds.value = icodeStore.items.map((i) => i.id);
}

function toggleSelectIcodeInclude() {
    // Clear selections when user disables include to avoid stale selection
    if (!includeIcode.value) selectedIcodeIds.value = [];
}

function toggleIcode(id: number) {
    const idx = selectedIcodeIds.value.indexOf(id);
    if (idx >= 0) selectedIcodeIds.value.splice(idx, 1);
    else selectedIcodeIds.value.push(id);
}

function toggleSelectAllDepartments() {
    if (allDepartmentsSelected.value) selectedDepartmentIds.value = [];
    else selectedDepartmentIds.value = deptStore.items.map((i) => i.id);
}

function toggleDepartment(id: number) {
    const idx = selectedDepartmentIds.value.indexOf(id);
    if (idx >= 0) selectedDepartmentIds.value.splice(idx, 1);
    else selectedDepartmentIds.value.push(id);
}

async function downloadJson(content: string, filename: string) {
    const filePath = await save({
        defaultPath: filename,
        filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (filePath) await writeTextFile(filePath, content);
}

async function handleExport() {
    exportError.value = "";
    isExporting.value = true;
    try {
        const options: ExportOptions = {
            include_pttype_groups: includePttype.value,
            pttype_group_ids:
                pttypeMode.value === "all"
                    ? null
                    : [...selectedPttypeIds.value],
            include_icode: includeIcode.value,
            icode_ids:
                icodeMode.value === "all" ? null : [...selectedIcodeIds.value],
            include_departments: includeDepartments.value,
            department_ids:
                departmentMode.value === "all"
                    ? null
                    : [...selectedDepartmentIds.value],
        };
        const json = await invoke<string>("export_settings", { options });
        const date = new Date().toISOString().slice(0, 10);
        await downloadJson(json, `setting_export_${date}.json`);
    } catch (e: unknown) {
        exportError.value = String(e);
    } finally {
        isExporting.value = false;
    }
}

// ── Import state ─────────────────────────────────────────────────────────────
const fileInput = ref<HTMLInputElement | null>(null);
const selectedFile = ref<File | null>(null);
const isImporting = ref(false);
const importError = ref("");
const importResult = ref<ImportSummary | null>(null);

function onFileChange(e: Event) {
    const input = e.target as HTMLInputElement;
    selectedFile.value = input.files?.[0] ?? null;
    importResult.value = null;
    importError.value = "";
}

async function reloadAllSettings() {
    await Promise.all([
        icodeStore.loadAll(),
        pttypeStore.loadAll(),
        deptStore.loadAll(),
    ]);
}

async function handleImport() {
    if (!selectedFile.value) return;
    importError.value = "";
    importResult.value = null;
    isImporting.value = true;
    try {
        const text = await selectedFile.value.text();
        const result = await invoke<ImportSummary>("import_settings", {
            data: text,
        });
        importResult.value = result;
        await reloadAllSettings();
    } catch (e: unknown) {
        importError.value = String(e);
    } finally {
        isImporting.value = false;
    }
}

onMounted(reloadAllSettings);
</script>

<template>
    <div class="export-import-settings">
        <div class="card mb-4">
            <div class="card-header">
                <Download :size="18" style="color: var(--brand-orange)" />
                ส่งออกการตั้งค่า
            </div>
            <div class="card-body">
                <div class="section-label">เลือกส่วนที่ต้องการส่งออก</div>

                <label class="check-row">
                    <input type="checkbox" v-model="includePttype" />
                    <span>สิทธิการรักษา (pttype groups)</span>
                </label>

                <div v-if="includePttype" class="mt-3">
                    <div class="radio-row">
                        <label class="radio-opt">
                            <input
                                type="radio"
                                v-model="pttypeMode"
                                value="all"
                            />
                            <span>ทั้งหมด</span>
                        </label>
                        <label class="radio-opt">
                            <input
                                type="radio"
                                v-model="pttypeMode"
                                value="select"
                            />
                            <span>เลือกเอง</span>
                        </label>
                    </div>

                    <div
                        v-if="pttypeMode === 'select'"
                        class="selection-box mt-3"
                    >
                        <div class="selection-header">
                            <span class="text-sm text-muted"
                                >เลือกแล้ว {{ selectedPttypeIds.length }} /
                                {{ pttypeStore.groups.length }} รายการ</span
                            >
                            <button
                                class="btn btn-ghost btn-sm"
                                @click="toggleSelectAllPttypes"
                            >
                                <component
                                    :is="
                                        allPttypesSelected
                                            ? CheckSquare
                                            : Square
                                    "
                                    :size="14"
                                />
                                {{
                                    allPttypesSelected
                                        ? "ยกเลิกทั้งหมด"
                                        : "เลือกทั้งหมด"
                                }}
                            </button>
                        </div>
                        <div class="selection-list">
                            <label
                                v-for="g in pttypeStore.groups"
                                :key="g.id"
                                class="selection-row"
                                :class="{
                                    selected: selectedPttypeIds.includes(g.id),
                                }"
                            >
                                <input
                                    type="checkbox"
                                    :value="g.id"
                                    :checked="selectedPttypeIds.includes(g.id)"
                                    @change="togglePttype(g.id)"
                                />
                                <span class="selection-name">{{
                                    g.alias
                                }}</span>
                                <span
                                    class="text-muted text-sm"
                                    style="margin-left: 8px"
                                    >{{ g.hipdata_code }}</span
                                >
                            </label>
                            <div
                                v-if="pttypeStore.groups.length === 0"
                                class="empty-msg"
                            >
                                ยังไม่มีรายการสิทธิการรักษา
                            </div>
                        </div>
                    </div>
                </div>

                <div class="divider" />

                <label class="check-row">
                    <input
                        type="checkbox"
                        v-model="includeIcode"
                        @change="toggleSelectIcodeInclude"
                    />
                    <span>รายการ icode</span>
                </label>

                <div v-if="includeIcode" class="mt-3">
                    <div class="radio-row">
                        <label class="radio-opt">
                            <input
                                type="radio"
                                v-model="icodeMode"
                                value="all"
                            />
                            <span>ทั้งหมด</span>
                        </label>
                        <label class="radio-opt">
                            <input
                                type="radio"
                                v-model="icodeMode"
                                value="select"
                            />
                            <span>เลือกเอง</span>
                        </label>
                    </div>

                    <div
                        v-if="icodeMode === 'select'"
                        class="selection-box mt-3"
                    >
                        <div class="selection-header">
                            <span class="text-sm text-muted"
                                >เลือกแล้ว {{ selectedIcodeIds.length }} /
                                {{ icodeStore.items.length }} รายการ</span
                            >
                            <button
                                class="btn btn-ghost btn-sm"
                                @click="toggleSelectAllIcodes"
                            >
                                <component
                                    :is="
                                        allIcodesSelected ? CheckSquare : Square
                                    "
                                    :size="14"
                                />
                                {{
                                    allIcodesSelected
                                        ? "ยกเลิกทั้งหมด"
                                        : "เลือกทั้งหมด"
                                }}
                            </button>
                        </div>
                        <div class="selection-list">
                            <label
                                v-for="item in icodeStore.items"
                                :key="item.id"
                                class="selection-row"
                                :class="{
                                    selected: selectedIcodeIds.includes(
                                        item.id,
                                    ),
                                }"
                            >
                                <input
                                    type="checkbox"
                                    :value="item.id"
                                    :checked="
                                        selectedIcodeIds.includes(item.id)
                                    "
                                    @change="toggleIcode(item.id)"
                                />
                                <span class="badge badge-gray font-mono">{{
                                    item.icode
                                }}</span>
                                <span class="selection-name">{{
                                    item.service_name
                                }}</span>
                                <span
                                    v-if="item.department"
                                    class="text-muted text-sm"
                                    >{{ item.department }}</span
                                >
                            </label>
                            <div
                                v-if="icodeStore.items.length === 0"
                                class="empty-msg"
                            >
                                ยังไม่มีรายการ icode
                            </div>
                        </div>
                    </div>
                </div>

                <div class="divider" />

                <label class="check-row">
                    <input type="checkbox" v-model="includeDepartments" />
                    <span>แผนกบริการ (department settings)</span>
                </label>

                <div v-if="includeDepartments" class="mt-3">
                    <div class="radio-row">
                        <label class="radio-opt">
                            <input
                                type="radio"
                                v-model="departmentMode"
                                value="all"
                            />
                            <span>ทั้งหมด</span>
                        </label>
                        <label class="radio-opt">
                            <input
                                type="radio"
                                v-model="departmentMode"
                                value="select"
                            />
                            <span>เลือกเอง</span>
                        </label>
                    </div>

                    <div
                        v-if="departmentMode === 'select'"
                        class="selection-box mt-3"
                    >
                        <div class="selection-header">
                            <span class="text-sm text-muted"
                                >เลือกแล้ว {{ selectedDepartmentIds.length }} /
                                {{ deptStore.items.length }} รายการ</span
                            >
                            <button
                                class="btn btn-ghost btn-sm"
                                @click="toggleSelectAllDepartments"
                            >
                                <component
                                    :is="
                                        allDepartmentsSelected
                                            ? CheckSquare
                                            : Square
                                    "
                                    :size="14"
                                />
                                {{
                                    allDepartmentsSelected
                                        ? "ยกเลิกทั้งหมด"
                                        : "เลือกทั้งหมด"
                                }}
                            </button>
                        </div>
                        <div class="selection-list">
                            <label
                                v-for="item in deptStore.items"
                                :key="item.id"
                                class="selection-row"
                                :class="{
                                    selected: selectedDepartmentIds.includes(
                                        item.id,
                                    ),
                                }"
                            >
                                <input
                                    type="checkbox"
                                    :value="item.id"
                                    :checked="
                                        selectedDepartmentIds.includes(item.id)
                                    "
                                    @change="toggleDepartment(item.id)"
                                />
                                <span class="badge badge-gray font-mono">{{
                                    item.depcode
                                }}</span>
                                <span class="selection-name">{{
                                    item.department
                                }}</span>
                                <span class="text-muted text-sm">{{
                                    item.is_enabled ? "เปิดใช้งาน" : "ปิดใช้งาน"
                                }}</span>
                            </label>
                            <div
                                v-if="deptStore.items.length === 0"
                                class="empty-msg"
                            >
                                ยังไม่มีรายการแผนก
                            </div>
                        </div>
                    </div>
                </div>

                <div class="divider" />

                <div class="action-row">
                    <button
                        class="btn btn-primary"
                        @click="handleExport"
                        :disabled="isExporting"
                    >
                        <Download :size="16" />
                        {{ isExporting ? "กำลังส่งออก..." : "ส่งออก (.json)" }}
                    </button>
                </div>

                <div v-if="exportError" class="alert alert-error mt-2">
                    <AlertCircle :size="16" />{{ exportError }}
                </div>
            </div>
        </div>

        <div class="card">
            <div class="card-header">
                <Upload :size="18" style="color: var(--brand-orange)" />
                นำเข้าการตั้งค่า
            </div>
            <div class="card-body">
                <p class="text-sm text-muted mb-3">
                    นำเข้าจากไฟล์
                    <code>.json</code>
                    ที่ส่งออกจากโปรแกรมนี้เท่านั้น<br />รองรับการนำเข้าสิทธิการรักษา,
                    รายการ icode และแผนกบริการ
                    โดยไม่รวมการตั้งค่าเชื่อมต่อฐานข้อมูล
                </p>

                <input
                    ref="fileInput"
                    type="file"
                    accept=".json"
                    style="display: none"
                    @change="onFileChange"
                />
                <div class="file-row">
                    <button
                        class="btn btn-secondary"
                        @click="fileInput?.click()"
                    >
                        <FileJson :size="16" /> เลือกไฟล์
                    </button>
                    <span v-if="selectedFile" class="file-name">{{
                        selectedFile.name
                    }}</span>
                    <span v-else class="text-muted text-sm"
                        >ยังไม่ได้เลือกไฟล์</span
                    >
                </div>

                <div class="divider" />

                <div class="action-row">
                    <button
                        class="btn btn-primary"
                        @click="handleImport"
                        :disabled="!selectedFile || isImporting"
                    >
                        <Upload :size="16" />
                        {{ isImporting ? "กำลังนำเข้า..." : "นำเข้า" }}
                    </button>
                </div>

                <div v-if="importError" class="alert alert-error mt-2">
                    <AlertCircle :size="16" />{{ importError }}
                </div>

                <div v-if="importResult" class="alert alert-success mt-2">
                    <Check :size="16" />
                    <div>
                        <div style="font-weight: 600; margin-bottom: 4px">
                            นำเข้าสำเร็จ
                        </div>
                        <div class="text-sm">
                            สิทธิการรักษา: เพิ่มใหม่
                            <strong>{{
                                importResult.pttype_groups_added
                            }}</strong>
                            รายการ, ข้าม
                            <strong>{{
                                importResult.pttype_groups_skipped
                            }}</strong>
                            รายการ
                        </div>
                        <div class="text-sm">
                            icode: เพิ่มใหม่
                            <strong>{{
                                importResult.icode_configs_added
                            }}</strong>
                            รายการ, อัปเดต
                            <strong>{{
                                importResult.icode_configs_updated
                            }}</strong>
                            รายการ
                        </div>
                        <div class="text-sm">
                            แผนกบริการ: เพิ่มใหม่
                            <strong>{{
                                importResult.department_configs_added
                            }}</strong>
                            รายการ, อัปเดต
                            <strong>{{
                                importResult.department_configs_updated
                            }}</strong>
                            รายการ
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
.export-import-settings {
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
.mt-3 {
    margin-top: 12px;
}
.mb-3 {
    margin-bottom: 12px;
}
.section-label {
    font-size: 13px;
    font-weight: 600;
    color: var(--slate);
    margin-bottom: 8px;
}
.check-row {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    font-size: 14px;
}
.radio-row {
    display: flex;
    gap: 20px;
}
.radio-opt {
    display: flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
    font-size: 14px;
}
.action-row {
    display: flex;
    gap: 8px;
}
.file-row {
    display: flex;
    align-items: center;
    gap: 12px;
}
.file-name {
    font-size: 13px;
    font-weight: 500;
    color: var(--slate);
}
.selection-box {
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    overflow: hidden;
}
.selection-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    background: var(--surface-soft);
    border-bottom: 1px solid var(--border);
}
.selection-list {
    max-height: 260px;
    overflow-y: auto;
}
.selection-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    cursor: pointer;
    font-size: 13px;
    transition: background 0.1s;
}
.selection-row:hover {
    background: var(--surface-soft);
}
.selection-row.selected {
    background: #fff7ed;
}
.selection-row input[type="checkbox"] {
    flex-shrink: 0;
}
.selection-name {
    flex: 1;
}
.empty-msg {
    padding: 20px;
    text-align: center;
    color: var(--muted);
    font-size: 13px;
}
</style>
