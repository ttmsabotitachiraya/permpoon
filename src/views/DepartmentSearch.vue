<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useConnectionStore } from "../stores/connectionStore";
import { usePttypeStore } from "../stores/pttypeStore";
import { useDepartmentStore } from "../stores/departmentStore";
import {
    Search,
    User,
    Printer,
    AlertCircle,
    CheckSquare,
    Square,
    Info,
    X,
    Building2,
} from "lucide-vue-next";
import type { PatientInfo } from "../types/patient";
import type { RecommendationItem } from "../types/icode";

interface PatientWithRecs {
    hn: string;
    fname: string;
    lname: string;
    cid: string;
    pttype: string;
    pttype_name: string;
    hipdata_code: string;
    dob: string;
    sex: string;
    age: number;
    vn?: string;
    recommendations: RecommendationItem[];
}

const connStore = useConnectionStore();
const pttypeStore = usePttypeStore();
const deptStore = useDepartmentStore();

const getTodayDate = () => {
    const d = new Date();
    const year = d.getFullYear();
    const month = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    return `${year}-${month}-${day}`;
};
const processDate = ref(getTodayDate());
const patients = ref<PatientInfo[]>([]);
const patientQuery = ref("");
const searchError = ref("");
const isSearching = ref(false);
const selectedPatient = ref<PatientInfo | null>(null);
const recommendations = ref<RecommendationItem[]>([]);
const selectedIcodes = ref<Set<string>>(new Set());
const recError = ref("");
const isLoadingRec = ref(false);
const showRecModal = ref(false);

onMounted(async () => {
    await pttypeStore.loadAll();
    await deptStore.loadAll();
});

function getPttypeAlias(p: PatientInfo) {
    return p.pttype_name || "-";
}

function hasRecommendations(hn: string): boolean {
    const patient = patients.value.find(p => p.hn === hn);
    return !!(patient && patient.recommendations && patient.recommendations.length > 0);
}

function getPttypeName(p: PatientInfo) {
    return (
        pttypeStore.groups.find((g) => g.hipdata_code === p.hipdata_code)
            ?.alias ||
        p.pttype_name ||
        p.pttype
    );
}

function getPttypeTooltip(p: PatientInfo) {
    return p.pttype_name || p.pttype;
}

function formatSex(sex: string): string {
    const s = sex.toUpperCase();
    if (s === "M" || s === "1") return "ชาย";
    if (s === "F" || s === "2") return "หญิง";
    return sex;
}

function formatDate(dateStr: string): string {
    if (!dateStr) return "-";
    const d = new Date(dateStr);
    const day = d.getDate().toString().padStart(2, "0");
    const month = (d.getMonth() + 1).toString().padStart(2, "0");
    const year = d.getFullYear() + 543;
    return `${day}/${month}/${year}`;
}

const enabledDepcodes = computed(() =>
    deptStore.items.filter((d) => d.is_enabled).map((d) => d.depcode),
);

const enabledDeptNames = computed(() =>
    deptStore.items
        .filter((d) => d.is_enabled)
        .map((d) => d.department)
        .join(", "),
);

const enabledDeptNamesFirst2 = computed(() =>
    deptStore.items
        .filter((d) => d.is_enabled)
        .slice(0, 2)
        .map((d) => d.department)
        .join(", "),
);

const useDefaultDepartment = computed(() => {
    return deptStore.items.length === 0 || enabledDepcodes.value.length === 0;
});

const filteredPatients = computed(() => {
    if (!patientQuery.value.trim()) return patients.value;
    const q = patientQuery.value.trim().toLowerCase();
    return patients.value.filter((p) => {
        if (p.hn.toLowerCase().includes(q)) return true;
        if (p.cid && p.cid.includes(q)) return true;
        if (p.fname.toLowerCase().includes(q)) return true;
        if (p.lname.toLowerCase().includes(q)) return true;
        return false;
    });
});

async function doSearch() {
    if (!connStore.isConnected) {
        searchError.value =
            "ยังไม่ได้เชื่อมต่อฐานข้อมูล กรุณาตั้งค่าที่เมนู 'การเชื่อมต่อ'";
        return;
    }
    isSearching.value = true;
    searchError.value = "";
    patients.value = [];
    selectedPatient.value = null;
    recommendations.value = [];
    selectedIcodes.value = new Set();

    try {
        const depcodes = useDefaultDepartment.value ? [] : enabledDepcodes.value;
        const results = await invoke<PatientWithRecs[]>(
            "search_patients_with_recommendations",
            {
                config: connStore.config,
                depcodes,
                processDate: processDate.value,
            },
        );

        patients.value = results.map(p => ({
            hn: p.hn,
            fname: p.fname,
            lname: p.lname,
            cid: p.cid,
            pttype: p.pttype,
            pttype_name: p.pttype_name,
            hipdata_code: p.hipdata_code,
            dob: p.dob,
            sex: p.sex,
            age: p.age,
            vn: p.vn || "",
            recommendations: p.recommendations || [],
        }));

        if (results.length === 0) {
            searchError.value = "ไม่พบผู้ป่วยในวันที่เลือก";
        }
    } catch (e: any) {
        searchError.value = String(e);
    } finally {
        isSearching.value = false;
    }
}

async function openRecommendations(patient: PatientInfo) {
    selectedPatient.value = patient;
    recommendations.value = patient.recommendations || [];
    selectedIcodes.value = new Set();
    recError.value = "";
    showRecModal.value = true;
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

const sortedRecommendations = computed(() => {
    return [...recommendations.value].sort((a, b) => {
        const deptA = a.department || "";
        const deptB = b.department || "";
        if (deptA < deptB) return -1;
        if (deptA > deptB) return 1;
        return a.service_name.localeCompare(b.service_name, "th");
    });
});

function closeRecModal() {
    showRecModal.value = false;
    selectedPatient.value = null;
}

async function printSlip() {
    if (selectedItems.value.length === 0 || !selectedPatient.value) return;
    const p = selectedPatient.value!;

    const tableRows = selectedItems.value
        .map(
            (item, i) => `
        <tr>
            <td class="td-num">${i + 1}</td>
            <td class="td-service">${item.service_name}</td>
            <td class="td-dept">${item.department || "-"}</td>
            <td class="td-note"></td>
        </tr>`,
        )
        .join("");

    const bodyContent = `
    <div class="slip-page">

      <div class="slip-header">
        <div class="slip-logo-wrap">
          <svg width="48" height="48" viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
            <defs>
              <linearGradient id="plg1" x1="0%" y1="0%" x2="0%" y2="100%">
                <stop offset="0%" style="stop-color:#f97316;stop-opacity:1"/>
                <stop offset="100%" style="stop-color:#fdba74;stop-opacity:1"/>
              </linearGradient>
              <linearGradient id="plg2" x1="0%" y1="0%" x2="0%" y2="100%">
                <stop offset="0%" style="stop-color:#fbbf24;stop-opacity:1"/>
                <stop offset="100%" style="stop-color:#fff7ed;stop-opacity:1"/>
              </linearGradient>
            </defs>
            <path d="M5 55 L50 72 L95 55 L50 38 Z" fill="url(#plg1)"/>
            <path d="M5 55 L5 70 L50 87 L50 72 Z" fill="url(#plg1)"/>
            <path d="M95 55 L95 70 L50 87 L50 72 Z" fill="url(#plg1)"/>
            <path d="M5 20 L50 37 L95 20 L50 3 Z" fill="url(#plg2)"/>
            <path d="M5 20 L5 35 L50 52 L50 37 Z" fill="url(#plg2)"/>
            <path d="M95 20 L95 35 L50 52 L50 37 Z" fill="url(#plg2)"/>
            <text x="50" y="98" font-family="Arial, sans-serif" font-size="13" text-anchor="middle" fill="#ea580c" font-weight="900" letter-spacing="2">P O O N</text>
          </svg>
          <div class="slip-brand">
            <span class="slip-brand-name">PermPoon</span>
            <span class="slip-brand-tagline">แนะนำบริการเพิ่มพูลรายได้</span>
          </div>
        </div>
        <div class="slip-title-wrap">
          <div class="slip-title">ใบแนะนำบริการ</div>
        </div>
      </div>

      <div class="slip-divider"></div>

      <div class="slip-section">
        <div class="slip-section-label">ข้อมูลส่วนตัว</div>
        <div class="slip-info-box">
          <div class="slip-row">
            <div class="slip-field half">
              <span class="slip-lbl">HN</span>
              <span class="slip-val mono">${p.hn}</span>
            </div>
            <div class="slip-field half">
              <span class="slip-lbl">วันที่</span>
              <span class="slip-val">${formatDate(processDate.value)}</span>
            </div>
          </div>
          <div class="slip-field">
            <span class="slip-lbl">ชื่อ-นามสกุล</span>
            <span class="slip-val">${p.fname} ${p.lname}</span>
          </div>
          <div class="slip-field" style="margin-bottom:0">
            <span class="slip-lbl">สิทธิการรักษา</span>
            <span class="slip-val">${getPttypeAlias(p)}</span>
          </div>
        </div>
      </div>

      <div class="slip-section">
        <div class="slip-section-label">บริการที่แนะนำ</div>
        <table class="slip-table">
          <thead>
            <tr>
              <th class="th-num">ลำดับ</th>
              <th class="th-service">บริการ</th>
              <th class="th-dept">แผนก</th>
              <th class="th-note">หมายเหตุ</th>
            </tr>
          </thead>
          <tbody>
            ${tableRows}
          </tbody>
        </table>
      </div>

      <div class="slip-footer">
        <div class="slip-footer-line"></div>
        <p>กรุณานำใบนี้ไปรับบริการที่จุดให้บริการ</p>
      </div>

    </div>
    `;

    const printCss = `
      @import url('https://fonts.googleapis.com/css2?family=Sarabun:wght@400;500;600;700&display=swap');

      @page { size: A5 portrait; margin: 10mm 12mm; }

      *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }

      body {
        font-family: 'Sarabun', 'TH SarabunNew', Arial, sans-serif;
        font-size: 14pt;
        line-height: 1.5;
        color: #1e293b;
        background: #ffffff;
        -webkit-print-color-adjust: exact;
        print-color-adjust: exact;
      }

      #__print_slip__ { display: none; }

      @media print {
        #app { display: none !important; }
        #__print_slip__ { display: block !important; }
      }

      .slip-page { width: 100%; }

      .slip-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 8pt;
      }
      .slip-logo-wrap {
        display: flex;
        align-items: center;
        gap: 8pt;
      }
      .slip-brand { display: flex; flex-direction: column; line-height: 1.2; }
      .slip-brand-name {
        font-size: 20pt;
        font-weight: 700;
        color: #ea580c;
        letter-spacing: -0.3pt;
      }
      .slip-brand-tagline {
        font-size: 9pt;
        color: #94a3b8;
        margin-top: 1pt;
      }
      .slip-title-wrap { text-align: right; }
      .slip-title {
        display: inline-block;
        font-size: 16pt;
        font-weight: 700;
        color: #ea580c;
        background: #fff7ed;
        border: 1.5pt solid #f97316;
        border-radius: 6pt;
        padding: 4pt 12pt;
      }

      .slip-divider {
        height: 2.5pt;
        background: linear-gradient(90deg, #f97316 0%, #fbbf24 60%, #fff7ed 100%);
        border-radius: 2pt;
        margin: 8pt 0 12pt;
      }

      .slip-section { margin-bottom: 12pt; }
      .slip-section-label {
        font-size: 12pt;
        font-weight: 700;
        color: #ea580c;
        text-transform: uppercase;
        letter-spacing: 0.4pt;
        border-left: 3pt solid #f97316;
        padding-left: 6pt;
        margin-bottom: 6pt;
      }

      .slip-info-box {
        border: 1pt solid #e2e8f0;
        border-radius: 6pt;
        padding: 8pt 12pt;
        background: #fafbfc;
      }
      .slip-row {
        display: flex;
        gap: 16pt;
        margin-bottom: 6pt;
      }
      .slip-field {
        display: flex;
        align-items: baseline;
        gap: 6pt;
        margin-bottom: 6pt;
      }
      .slip-field.half { flex: 1; margin-bottom: 0; }
      .slip-lbl {
        font-size: 11pt;
        font-weight: 600;
        color: #64748b;
        white-space: nowrap;
      }
      .slip-lbl::after { content: ':'; }
      .slip-val {
        font-size: 13pt;
        font-weight: 500;
        color: #0f172a;
      }
      .mono {
        font-family: 'Courier New', monospace;
        font-weight: 700;
        color: #ea580c;
        font-size: 14pt;
      }

      .slip-table {
        width: 100%;
        border-collapse: collapse;
        font-size: 12pt;
      }
      .slip-table thead tr {
        background: #fff7ed;
      }
      .slip-table th {
        padding: 6pt 8pt;
        font-size: 12pt;
        font-weight: 700;
        color: #c2410c;
        text-align: left;
        border: 1pt solid #fed7aa;
      }
      .slip-table td {
        padding: 6pt 8pt;
        border: 1pt solid #e2e8f0;
        color: #1e293b;
        vertical-align: middle;
      }
      .slip-table tbody tr:nth-child(even) td { background: #fff7ed; }
      .th-num, .td-num  { width: 30pt; text-align: center; }
      .th-dept, .td-dept { width: 80pt; }
      .th-note, .td-note { width: 60pt; }
      .th-service, .td-service { }

      .slip-footer { margin-top: 16pt; text-align: center; }
      .slip-footer-line {
        height: 1pt;
        background: #e2e8f0;
        margin-bottom: 6pt;
      }
      .slip-footer p { font-size: 10pt; color: #94a3b8; }
    `;

    const printContainer = document.createElement("div");
    printContainer.id = "__print_slip__";
    printContainer.innerHTML = bodyContent;

    const printStyleEl = document.createElement("style");
    printStyleEl.id = "__print_style__";
    printStyleEl.innerHTML = printCss;

    document.head.appendChild(printStyleEl);
    document.body.appendChild(printContainer);
    await new Promise<void>((r) => setTimeout(r, 80));
    try {
        await invoke("plugin:webview|print");
    } finally {
        setTimeout(() => {
            printStyleEl.remove();
            printContainer.remove();
        }, 3000);
    }
}

async function handlePrintFromModal() {
    await printSlip();
    closeRecModal();
}

function clearPatientQuery() {
    patientQuery.value = "";
}
</script>

<template>
    <div class="dept-search">
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
                <button
                    class="btn btn-primary btn-sm search-btn"
                    @click="doSearch"
                    :disabled="isSearching"
                >
                    <Search :size="15" />
                    {{ isSearching ? "กำลังค้นหา..." : "ค้นหา" }}
                </button>
                <div style="margin-left: auto; display: flex; gap: 8px; align-items: flex-end;">
                    <div class="search-query-wrap">
                        <label class="bar-label">ค้นหาผู้ป่วย (HN / CID / ชื่อ)</label>
                        <div class="input-with-clear">
                            <input
                                v-model="patientQuery"
                                type="text"
                                class="form-input bar-input"
                                placeholder="เช่น 0000001, 1234567890123, สมชาย"
                            />
                            <button
                                v-if="patientQuery"
                                class="clear-btn"
                                @click="clearPatientQuery"
                                title="ล้างข้อมูล"
                            >
                                <X :size="14" />
                            </button>
                        </div>
                    </div>
                    <div
                        v-if="enabledDepcodes.length > 0"
                        class="dept-names-display"
                        :title="enabledDeptNames"
                    >
                        <Building2 :size="12" />
                        <span v-if="enabledDepcodes.length === 1">
                            {{ enabledDeptNames }}
                        </span>
                        <span v-else-if="enabledDepcodes.length === 2">
                            {{ enabledDeptNames }}
                        </span>
                        <span v-else>
                            {{ enabledDeptNamesFirst2 }}{{ enabledDepcodes.length - 2 > 0 ? ', +' + (enabledDepcodes.length - 2) : '' }}
                        </span>
                    </div>
                </div>
            </div>
            <div v-if="searchError" class="alert alert-error bar-error">
                <AlertCircle :size="14" /> {{ searchError }}
            </div>
        </div>

        <div class="main-content">
            <div v-if="patients.length > 0" class="patient-list">
                <div class="list-header">
                    <User :size="15" style="color: var(--brand-orange)" />
                    พบผู้ป่วย <strong>{{ filteredPatients.length }}</strong> ราย
                    <span v-if="patientQuery" class="filter-note">
                        (กรองจาก {{ patients.length }} ราย)
                    </span>
                </div>
                <table class="table patient-table">
                    <thead>
                        <tr>
                            <th>HN</th>
                            <th>ชื่อ-นามสกุล</th>
                            <th>อายุ</th>
                            <th>เพศ</th>
                            <th>สิทธิ์</th>
                            <th style="width: 160px"></th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr v-for="p in filteredPatients" :key="p.hn">
                            <td>
                                <span class="font-mono">{{ p.hn }}</span>
                            </td>
                            <td>{{ p.fname }} {{ p.lname }}</td>
                            <td>{{ p.age }} ปี</td>
                            <td>{{ formatSex(p.sex) }}</td>
                            <td :title="getPttypeTooltip(p)">
                                <span class="pttype-name">{{ getPttypeName(p) }}</span>
                            </td>
                            <td>
                                <button
                                    v-if="hasRecommendations(p.hn)"
                                    class="btn-recommend btn-sm"
                                    @click="openRecommendations(p)"
                                >
                                    แนะนำบริการ ({{ p.recommendations?.length || 0 }})
                                </button>
                                <button
                                    v-else
                                    class="btn-recommend-disabled btn-sm"
                                    disabled
                                >
                                    แนะนำบริการ
                                </button>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>

            <div v-else-if="!isSearching && !searchError" class="empty-state">
                <Building2
                    :size="48"
                    style="color: var(--muted); margin-bottom: 16px"
                />
                <p style="color: var(--slate); font-size: 16px">
                    เลือกวันที่และกดค้นหาเพื่อดูรายชื่อผู้ป่วย
                </p>
            </div>
        </div>

        <div
            v-if="showRecModal"
            class="modal-overlay"
            @click.self="closeRecModal"
        >
            <div class="modal rec-modal">
                <div class="modal-header">
                    <div
                        style="display: flex; align-items: center; gap: 8px"
                    >
                        <CheckSquare
                            :size="16"
                            style="color: var(--brand-orange)"
                        />
                        <span>แนะนำบริการ</span>
                    </div>
                    <button class="btn btn-ghost btn-sm" @click="closeRecModal">
                        <X :size="14" />
                    </button>
                </div>
                <div class="modal-body">
                    <div v-if="selectedPatient" class="patient-info-box">
                        <div class="patient-row">
                            <span class="lbl">HN:</span>
                            <span class="val font-mono">{{
                                selectedPatient.hn
                            }}</span>
                        </div>
                        <div class="patient-row">
                            <span class="lbl">ชื่อ:</span>
                            <span class="val"
                                >{{ selectedPatient.fname }}
                                {{ selectedPatient.lname }}</span
                            >
                        </div>
                        <div class="patient-row">
                            <span class="lbl">สิทธิ์:</span>
                            <span class="val">{{
                                getPttypeAlias(selectedPatient)
                            }}</span>
                        </div>
                    </div>

                    <div
                        v-if="isLoadingRec"
                        class="loading-state"
                    >
                        <div class="spinner"></div>
                        <span>กำลังโหลดรายการแนะนำ...</span>
                    </div>
                    <div v-else-if="recError" class="alert alert-error">
                        <AlertCircle :size="16" /> {{ recError }}
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
                    </div>
                    <template v-else>
                        <div class="rec-actions">
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
                            <span class="select-count">
                                เลือก {{ selectedIcodes.size }} /
                                {{ recommendations.length }} รายการ
                            </span>
                        </div>
                        <table class="table rec-table">
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
                                                selectedIcodes.has(item.icode)
                                            "
                                            :size="18"
                                            style="color: var(--brand-orange)"
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
                                        <span v-else class="text-muted">-</span>
                                    </td>
                                </tr>
                            </tbody>
                        </table>
                    </template>
                </div>
                <div class="modal-footer">
                    <button
                        class="btn-recommend"
                        @click="handlePrintFromModal"
                        :disabled="selectedIcodes.size === 0"
                    >
                        <Printer :size="15" />
                        พิมพ์สลิป ({{ selectedIcodes.size }} รายการ)
                    </button>
                    <button class="btn btn-ghost" @click="closeRecModal">
                        ปิด
                    </button>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
.dept-search {
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

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
.search-dept-info {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 12px;
    justify-content: flex-end;
}
.search-dept-wrap {
    flex-shrink: 0;
    min-width: 150px;
}
.search-query-wrap {
    flex: 1;
    max-width: 300px;
}
.dept-names-display {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--steel);
    background: var(--surface);
    padding: 0 10px;
    height: 34px;
    border-radius: var(--radius-md);
    border: 1px solid var(--hairline);
    max-width: 300px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    cursor: default;
}
.search-btn {
    height: 34px;
    padding: 0 16px;
}
.dept-info-badge {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--steel);
    background: var(--surface);
    padding: 4px 10px;
    border-radius: var(--radius-full);
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

.main-content {
    flex: 1;
    overflow-y: auto;
    min-width: 0;
    padding: 16px 20px;
}

.patient-list {
    background: white;
    border: 1px solid var(--hairline);
    border-radius: var(--radius-lg);
    overflow: hidden;
}
.list-header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 10px 16px;
    font-size: 14px;
    color: var(--charcoal);
    background: #f8fafc;
    border-bottom: 1px solid var(--hairline);
}
.filter-note {
    font-size: 12px;
    color: var(--steel);
    font-weight: normal;
}
.patient-table {
    font-size: 13px;
}
.patient-table th:nth-child(1) {
    width: 100px;
}
.patient-table th:nth-child(3) {
    width: 80px;
}
.patient-table th:nth-child(4) {
    width: 60px;
}
.patient-table th:nth-child(5) {
    min-width: 120px;
}
.patient-table th:nth-child(6) {
    width: 120px;
}
.pttype-badge {
    font-size: 11px;
    padding: 2px 8px;
    background: var(--surface);
    border: 1px solid var(--hairline);
    border-radius: var(--radius-full);
}
.pttype-name {
    font-size: 13px;
    color: var(--charcoal);
    max-width: 150px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    display: block;
}

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

.loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 32px;
    color: var(--slate);
}

.modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 3000;
    padding: 20px;
}
.rec-modal {
    position: relative;
    background: white;
    border-radius: 8px;
    width: 700px;
    max-width: 96vw;
    max-height: 90vh;
    overflow: hidden;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.25);
    display: flex;
    flex-direction: column;
}
.modal-header {
    padding: 12px 16px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-bottom: 1px solid var(--hairline);
}
.modal-body {
    padding: 16px;
    overflow-y: auto;
    flex: 1;
}
.modal-footer {
    padding: 12px 16px;
    display: flex;
    gap: 8px;
    justify-content: flex-end;
    border-top: 1px solid var(--hairline);
}

.patient-info-box {
    background: var(--surface);
    border: 1px solid var(--hairline);
    border-radius: var(--radius-md);
    padding: 12px;
    margin-bottom: 16px;
}
.patient-row {
    display: flex;
    gap: 12px;
    margin-bottom: 4px;
}
.patient-row:last-child {
    margin-bottom: 0;
}
.patient-row .lbl {
    font-size: 12px;
    color: var(--steel);
    width: 50px;
    flex-shrink: 0;
}
.patient-row .val {
    font-size: 13px;
    color: var(--charcoal);
}

.rec-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 12px;
}
.select-count {
    margin-left: auto;
    font-size: 12px;
    color: var(--slate);
}

.rec-table {
    font-size: 13px;
}
.rec-table th:nth-child(1) {
    width: 40px;
}
.rec-table th:nth-child(3) {
    width: 150px;
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

.btn-recommend {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 5px 14px;
    min-width: 120px;
    font-size: 13px;
    font-weight: 500;
    color: white;
    background: linear-gradient(135deg, #f97316, #ea580c);
    border: none;
    border-radius: 6px;
    cursor: pointer;
    box-shadow: 0 2px 6px rgba(249,115,22,0.35);
    transition: all 0.15s ease;
    white-space: nowrap;
}
.btn-recommend:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    filter: none;
    box-shadow: none;
}

.btn-recommend-disabled {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 5px 14px;
    min-width: 120px;
    font-size: 13px;
    font-weight: 500;
    color: #94a3b8;
    background: #f1f5f9;
    border: none;
    border-radius: 6px;
    cursor: not-allowed;
    opacity: 0.7;
    white-space: nowrap;
}

.btn-no-rec {
    display: inline-flex;
    align-items: center;
    padding: 6px 12px;
    font-size: 12px;
    font-weight: 500;
    color: #94a3b8;
    background: #f1f5f9;
    border-radius: 6px;
    cursor: not-allowed;
    user-select: none;
}

.btn-loading {
    display: inline-flex;
    align-items: center;
    padding: 6px 12px;
    font-size: 12px;
    color: #94a3b8;
    background: #f8fafc;
    border: 1px dashed #cbd5e1;
    border-radius: 6px;
}

.input-with-clear {
    position: relative;
    display: flex;
    align-items: center;
}
.input-with-clear .bar-input {
    padding-right: 32px;
}
.clear-btn {
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
.clear-btn:hover {
    background: #cbd5e1;
    color: #475569;
}
</style>