<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useConnectionStore } from "../stores/connectionStore";
import { usePttypeStore } from "../stores/pttypeStore";
import {
    Search,
    User,
    Users,
    Printer,
    AlertCircle,
    CheckSquare,
    Square,
    Info,
    X,
} from "lucide-vue-next";
import type { PatientInfo } from "../types/patient";
import type { RecommendationItem } from "../types/icode";
import { createRecommendationSlipPdfBase64 } from "../utils/recommendationSlipPdf";

const connStore = useConnectionStore();
const pttypeStore = usePttypeStore();

onMounted(async () => {
    // โหลดรายชื่อ alias สิทธิการรักษาที่ผู้ใช้ตั้งไว้
    await pttypeStore.loadAll();
});

function getPttypeAlias(p: PatientInfo) {
    return (
        pttypeStore.groups.find((g) => g.hipdata_code === p.hipdata_code)
            ?.alias ||
        p.pttype_name ||
        p.pttype
    );
}

// State
const searchQuery = ref("");
const getTodayDate = () => {
    const d = new Date();
    const year = d.getFullYear();
    const month = String(d.getMonth() + 1).padStart(2, "0");
    const day = String(d.getDate()).padStart(2, "0");
    return `${year}-${month}-${day}`;
};
const processDate = ref(getTodayDate());
const patient = ref<PatientInfo | null>(null);
const searchResults = ref<PatientInfo[]>([]);
const recommendations = ref<RecommendationItem[]>([]);
const selectedIcodes = ref<Set<string>>(new Set());
const isSearching = ref(false);
const isLoadingRec = ref(false);
const searchError = ref("");
const recError = ref("");

// Format date for display
function formatDate(d: string) {
    if (!d) return "-";
    const [y, m, day] = d.split("-");
    return `${day}/${m}/${y}`;
}

function formatSex(sex: string) {
    return sex === "M" ? "ชาย" : sex === "F" ? "หญิง" : sex;
}

// Parse search query
function parseQuery(q: string) {
    const trimmed = q.trim();
    if (/^\d{1,7}$/.test(trimmed)) {
        return { hn: trimmed.padStart(7, "0") };
    }
    if (/^\d{13}$/.test(trimmed)) {
        return { cid: trimmed };
    }
    // If raw input starts with space/tab → user wants lname-only search
    if (q.startsWith(" ") || q.startsWith("\t")) {
        return { lname: trimmed };
    }
    const parts = trimmed.split(/\s+/);
    if (parts.length >= 2) {
        return { fname: parts[0], lname: parts.slice(1).join(" ") };
    }
    // Single word (no leading space) → fname search
    return { fname: trimmed };
}

async function doSearch() {
    if (!searchQuery.value.trim()) return;
    if (!connStore.isConnected) {
        searchError.value =
            "ยังไม่ได้เชื่อมต่อฐานข้อมูล กรุณาตั้งค่าที่เมนู 'การเชื่อมต่อ'";
        return;
    }
    isSearching.value = true;
    searchError.value = "";
    patient.value = null;
    searchResults.value = [];
    recommendations.value = [];
    selectedIcodes.value = new Set();

    try {
        const query = parseQuery(searchQuery.value);
        const results = await invoke<PatientInfo[]>("lookup_patient", {
            config: connStore.config,
            query,
            processDate: processDate.value,
        });
        if (results.length === 0) {
            searchError.value = "ไม่พบข้อมูลผู้ป่วย";
        } else if (results.length === 1) {
            patient.value = results[0];
            await loadRecommendations();
        } else {
            // พบหลายคน — ให้ผู้ใช้เลือก (แสดงสูงสุด 50 รายการ)
            searchResults.value = results.slice(0, 50);
        }
    } catch (e: any) {
        searchError.value = String(e);
    } finally {
        isSearching.value = false;
    }
}

async function selectPatient(p: PatientInfo) {
    patient.value = p;
    searchResults.value = [];
    recommendations.value = [];
    selectedIcodes.value = new Set();
    await loadRecommendations();
}

async function loadRecommendations() {
    if (!patient.value) return;
    isLoadingRec.value = true;
    recError.value = "";
    try {
        const items = await invoke<RecommendationItem[]>(
            "get_recommendations",
            {
                config: connStore.config,
                hn: patient.value.hn,
                processDate: processDate.value,
            },
        );
        recommendations.value = items;
        // Fix 3: default ไม่เลือกทั้งหมด
        selectedIcodes.value = new Set();
    } catch (e: any) {
        recError.value = String(e);
    } finally {
        isLoadingRec.value = false;
    }
}

function toggleSelect(icode: string) {
    if (selectedIcodes.value.has(icode)) {
        selectedIcodes.value.delete(icode);
    } else {
        selectedIcodes.value.add(icode);
    }
    selectedIcodes.value = new Set(selectedIcodes.value);
}

function selectAll() {
    selectedIcodes.value = new Set(recommendations.value.map((i) => i.icode));
}

function clearAll() {
    selectedIcodes.value = new Set();
}

const selectedItems = computed(() =>
    recommendations.value.filter((i) => selectedIcodes.value.has(i.icode)),
);

// Sort recommendations so items with the same department are grouped together
const sortedRecommendations = computed(() => {
    return [...recommendations.value].sort((a, b) => {
        const deptA = a.department || "";
        const deptB = b.department || "";
        if (deptA < deptB) return -1;
        if (deptA > deptB) return 1;
        return a.service_name.localeCompare(b.service_name, "th");
    });
});

function buildSlipInput() {
    if (!patient.value || selectedItems.value.length === 0) return null;
    const p = patient.value;
    return {
        patient: {
            hn: p.hn,
            fname: p.fname,
            lname: p.lname,
        },
        items: selectedItems.value,
        processDateText: formatDate(processDate.value),
        pttypeText: getPttypeAlias(p),
    };
}

async function openSlipForPrinting() {
    const input = buildSlipInput();
    if (!input) return;

    recError.value = "";
    try {
        const pdfBase64 = await createRecommendationSlipPdfBase64(input);
        await invoke("open_temp_pdf_with_viewer", {
            input: {
                file_name: `advice-slip-${input.patient.hn}-${processDate.value}.pdf`,
                pdf_base64: pdfBase64,
            },
        });
    } catch (e: unknown) {
        recError.value = `เปิดไฟล์เพื่อพิมพ์ไม่สำเร็จ: ${String(e)}`;
    }
}
</script>

<template>
    <div class="smart-advice">
        <!-- Compact top search bar -->
        <div class="search-bar">
            <div v-if="!connStore.isConnected" class="warn-inline">
                <AlertCircle :size="14" />
                ยังไม่ได้เชื่อมต่อ HOSxP
            </div>
            <div class="search-fields">
                <div class="search-date-wrap">
                    <label class="bar-label">วันที่ประมวลผล</label>
                    <input
                        v-model="processDate"
                        type="date"
                        class="form-input bar-input"
                    />
                </div>
                <div class="search-query-wrap">
                    <label class="bar-label"
                        >HN / เลขบัตรประชาชน / ชื่อ-นามสกุล</label
                    >
                    <div class="search-input-row">
                        <div class="input-with-clear" style="flex: 1">
                            <input
                                v-model="searchQuery"
                                type="text"
                                class="form-input bar-input"
                                style="width: 100%"
                                placeholder="เช่น 0000001, 1234567890123, สมชาย ใจดี"
                                @keydown.enter="doSearch"
                            />
                            <button
                                v-if="searchQuery"
                                class="clear-btn-icon"
                                @click="searchQuery = ''"
                                title="ล้างข้อมูล"
                            >
                                <X :size="14" />
                            </button>
                        </div>
                        <button
                            class="btn btn-primary btn-sm"
                            @click="doSearch"
                            :disabled="isSearching || !searchQuery.trim()"
                        >
                            <Search :size="15" />
                            {{ isSearching ? "กำลังค้นหา..." : "ตรวจสอบ" }}
                        </button>
                    </div>
                </div>
            </div>
            <div v-if="searchError" class="alert alert-error bar-error">
                <AlertCircle :size="14" /> {{ searchError }}
            </div>
        </div>

        <!-- Patient picker: shown when multiple results found -->
        <div v-if="searchResults.length > 1" class="patient-picker">
            <div class="picker-header">
                <Users :size="15" style="color: var(--brand-orange)" />
                พบผู้ป่วย <strong>{{ searchResults.length }}</strong> ราย —
                กรุณาเลือกผู้ป่วย
            </div>
            <div class="picker-list">
                <div
                    v-for="p in searchResults"
                    :key="p.hn"
                    class="picker-item"
                    @click="selectPatient(p)"
                >
                    <span class="picker-hn">{{ p.hn }}</span>
                    <span class="picker-name">{{ p.fname }} {{ p.lname }}</span>
                    <span class="picker-meta">
                        อายุ {{ p.age }} ปี · {{ formatSex(p.sex) }} ·
                        {{ getPttypeAlias(p) }}
                    </span>
                </div>
            </div>
        </div>

        <!-- Main content: 2-column layout -->
        <div class="main-content">
            <!-- Left: Patient Info -->
            <div v-if="patient" class="left-panel">
                <div class="card">
                    <div class="card-header">
                        <User :size="16" style="color: var(--brand-orange)" />
                        ข้อมูลผู้ป่วย
                    </div>
                    <div class="card-body patient-body">
                        <!-- Row 1: HN + สิทธิ์การรักษา -->
                        <div class="patient-row-inline">
                            <div class="patient-field-inline">
                                <span class="field-label">HN</span>
                                <span class="field-value font-mono">{{
                                    patient.hn
                                }}</span>
                            </div>
                            <div class="patient-field-inline">
                                <span class="field-label">สิทธิการรักษา</span>
                                <span
                                    class="field-value"
                                    :title="patient.pttype_name"
                                    >{{ getPttypeAlias(patient) }}</span
                                >
                            </div>
                        </div>
                        <!-- Row 2: ชื่อ-นามสกุล -->
                        <div class="patient-field">
                            <span class="field-label">ชื่อ-นามสกุล</span>
                            <span class="field-value"
                                >{{ patient.fname }} {{ patient.lname }}</span
                            >
                        </div>
                        <!-- Row 3: เลขบัตรประชาชน -->
                        <div class="patient-field">
                            <span class="field-label">เลขบัตรประชาชน</span>
                            <span class="field-value font-mono">{{
                                patient.cid || "-"
                            }}</span>
                        </div>
                        <!-- Row 4: อายุ + เพศ -->
                        <div class="patient-row-inline">
                            <div class="patient-field-inline">
                                <span class="field-label">อายุ</span>
                                <span class="field-value"
                                    >{{ patient.age }} ปี</span
                                >
                            </div>
                            <div class="patient-field-inline">
                                <span class="field-label">เพศ</span>
                                <span class="field-value">{{
                                    formatSex(patient.sex)
                                }}</span>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Print button below patient card -->
                <div v-if="recommendations.length > 0" class="print-area">
                    <div class="select-count">
                        เลือก {{ selectedIcodes.size }} /
                        {{ recommendations.length }} รายการ
                    </div>
                    <button
                        class="btn btn-primary"
                        style="width: 100%"
                        @click="openSlipForPrinting"
                        :disabled="selectedIcodes.size === 0"
                    >
                        <Printer :size="15" />
                        พิมพ์สลิปบริการ ({{ selectedIcodes.size }} รายการ)
                    </button>
                </div>
            </div>

            <!-- Right: Recommendations -->
            <div v-if="patient" class="right-panel">
                <div class="card rec-card">
                    <div
                        class="card-header"
                        style="justify-content: space-between"
                    >
                        <div
                            style="display: flex; align-items: center; gap: 8px"
                        >
                            <CheckSquare
                                :size="16"
                                style="color: var(--brand-orange)"
                            />
                            บริการที่แนะนำ
                            <span
                                v-if="recommendations.length"
                                class="badge badge-orange"
                            >
                                {{ recommendations.length }} รายการ
                            </span>
                        </div>
                        <div
                            v-if="recommendations.length"
                            style="display: flex; gap: 8px"
                        >
                            <button
                                class="btn btn-secondary btn-sm"
                                @click="selectAll"
                            >
                                เลือกทั้งหมด
                            </button>
                            <button
                                class="btn btn-ghost btn-sm"
                                @click="clearAll"
                            >
                                ยกเลิกทั้งหมด
                            </button>
                        </div>
                    </div>
                    <div
                        class="card-body"
                        style="padding: 0; overflow-y: auto; flex: 1"
                    >
                        <div v-if="isLoadingRec" class="loading-state">
                            <div class="spinner"></div>
                            <span>กำลังโหลดรายการแนะนำ...</span>
                        </div>
                        <div
                            v-else-if="recError"
                            class="alert alert-error"
                            style="margin: 16px"
                        >
                            <AlertCircle :size="16" />
                            {{ recError }}
                        </div>
                        <div
                            v-else-if="recommendations.length === 0"
                            class="empty-state"
                        >
                            <Info
                                :size="32"
                                style="color: var(--steel); margin-bottom: 8px"
                            />
                            <p>ไม่พบบริการที่แนะนำสำหรับคนไข้รายนี้</p>
                            <p
                                class="text-sm text-muted"
                                style="margin-top: 4px"
                            >
                                อาจเนื่องจากได้รับบริการแล้ววันนี้
                                หรือไม่ผ่านเงื่อนไข
                            </p>
                        </div>
                        <template v-else>
                            <table class="table">
                                <thead>
                                    <tr>
                                        <th style="width: 40px"></th>
                                        <th>ชื่อบริการ</th>
                                        <th>แผนก</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    <tr
                                        v-for="item in sortedRecommendations"
                                        :key="item.icode"
                                        class="rec-row"
                                        :class="{
                                            selected: selectedIcodes.has(
                                                item.icode,
                                            ),
                                        }"
                                        @click="toggleSelect(item.icode)"
                                    >
                                        <td>
                                            <CheckSquare
                                                v-if="
                                                    selectedIcodes.has(
                                                        item.icode,
                                                    )
                                                "
                                                :size="18"
                                                style="
                                                    color: var(--brand-orange);
                                                "
                                            />
                                            <Square
                                                v-else
                                                :size="18"
                                                style="color: var(--muted)"
                                            />
                                        </td>
                                        <td class="service-name">
                                            {{ item.service_name }}
                                        </td>
                                        <td>
                                            <span
                                                v-if="item.department"
                                                class="badge badge-gray"
                                                >{{ item.department }}</span
                                            >
                                            <span v-else class="text-muted"
                                                >-</span
                                            >
                                        </td>
                                    </tr>
                                </tbody>
                            </table>
                        </template>
                    </div>
                </div>
            </div>

            <!-- Empty state when no patient selected -->
            <div v-if="!patient" class="no-patient-state">
                <Search
                    :size="48"
                    style="color: var(--muted); margin-bottom: 16px"
                />
                <p style="color: var(--slate); font-size: 16px">
                    ค้นหาผู้ป่วยเพื่อดูบริการที่แนะนำ
                </p>
            </div>
        </div>
    </div>
</template>

<style scoped>
.smart-advice {
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

/* ===== Compact search bar ===== */
.search-bar {
    flex-shrink: 0;
    background: white;
    border-bottom: 1px solid var(--hairline);
    padding: 10px 20px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
}
.search-fields {
    display: flex;
    gap: 12px;
    align-items: flex-end;
}
.search-date-wrap {
    flex-shrink: 0;
    width: 170px;
}
.search-query-wrap {
    flex: 1;
}
.bar-label {
    font-size: 11px;
    font-weight: 500;
    color: var(--steel);
    display: block;
    margin-bottom: 3px;
}
.bar-input {
    height: 34px;
    font-size: 13px;
    padding: 0 8px;
}
.search-input-row {
    display: flex;
    gap: 8px;
}
.search-input-row .form-input {
    flex: 1;
}
.warn-inline {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: #b45309;
    background: #fef3c7;
    border: 1px solid #fde68a;
    border-radius: 6px;
    padding: 4px 10px;
    margin-bottom: 8px;
}
.bar-error {
    margin-top: 6px;
    padding: 6px 10px;
    font-size: 12px;
}

/* ===== Main 2-column layout ===== */
.main-content {
    flex: 1;
    display: flex;
    gap: 16px;
    padding: 16px 20px;
    overflow: hidden;
    min-height: 0;
}

.left-panel {
    width: 300px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 12px;
    overflow-y: auto;
}

.right-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow: hidden;
}

.rec-card {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-height: 0;
}

.patient-body {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 12px 14px;
}
.patient-field {
    display: flex;
    flex-direction: column;
    gap: 2px;
}
.patient-row-inline {
    display: flex;
    gap: 16px;
}
.patient-field-inline {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
}
.pttype-badge {
    cursor: default;
    width: fit-content;
}
.field-label {
    font-size: 11px;
    color: var(--steel);
    font-weight: 500;
}
.field-value {
    font-size: 13px;
    color: var(--charcoal);
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
}

.print-area {
    display: flex;
    flex-direction: column;
    gap: 8px;
}
.select-count {
    font-size: 12px;
    color: var(--slate);
    text-align: center;
}

/* ===== Rec table ===== */
.loading-state,
.empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 48px;
    color: var(--slate);
}
.spinner {
    width: 24px;
    height: 24px;
    border: 3px solid var(--hairline);
    border-top-color: var(--brand-orange);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    margin-bottom: 12px;
}
@keyframes spin {
    to {
        transform: rotate(360deg);
    }
}

.rec-row {
    cursor: pointer;
    transition: background 0.1s;
}
.rec-row.selected td {
    background: #fff7ed;
}
.service-name {
    font-weight: 500;
}

.no-patient-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
}

/* ===== Clear button ===== */
.input-with-clear {
    position: relative;
    display: flex;
    align-items: center;
}
.input-with-clear .bar-input {
    padding-right: 32px;
}
.clear-btn-icon {
    position: absolute;
    right: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border: none;
    background: #e2e8f0;
    border-radius: 50%;
    cursor: pointer;
    color: #64748b;
    transition: all 0.15s;
}
.clear-btn-icon:hover {
    background: #cbd5e1;
    color: #475569;
}

.clear-btn {
    color: var(--steel);
    border: 1px solid var(--hairline);
    border-radius: var(--radius-md);
    padding: 0 8px;
    height: 34px;
    flex-shrink: 0;
    transition:
        color 0.15s,
        background 0.15s;
}
.clear-btn:hover {
    color: var(--error);
    background: #fef2f2;
    border-color: #fecaca;
}

/* ===== Patient picker panel ===== */
.patient-picker {
    flex-shrink: 0;
    background: white;
    border-bottom: 1px solid var(--hairline);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
}
.picker-header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 20px;
    font-size: 13px;
    color: var(--charcoal);
    background: #fff7ed;
    border-bottom: 1px solid #fed7aa;
}
.picker-header strong {
    color: var(--brand-deep);
}
.picker-list {
    display: flex;
    flex-direction: column;
    max-height: 220px;
    overflow-y: auto;
}
.picker-item {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 9px 20px;
    cursor: pointer;
    border-bottom: 1px solid var(--hairline);
    transition: background 0.12s;
}
.picker-item:last-child {
    border-bottom: none;
}
.picker-item:hover {
    background: #fff7ed;
}
.picker-hn {
    font-family: "Geist Mono", "SF Mono", Menlo, Consolas, monospace;
    font-size: 12px;
    font-weight: 700;
    color: var(--brand-deep);
    width: 72px;
    flex-shrink: 0;
}
.picker-name {
    flex: 1;
    font-size: 13px;
    font-weight: 600;
    color: var(--charcoal);
}
.picker-meta {
    font-size: 12px;
    color: var(--slate);
    flex-shrink: 0;
}
.picker-pttype {
    flex-shrink: 0;
    font-size: 12px;
    color: var(--slate);
}
</style>
