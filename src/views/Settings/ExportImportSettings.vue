<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useIcodeStore } from '../../stores/icodeStore'
import {
  Download, Upload, FileJson, CheckSquare, Square, AlertCircle, Check
} from 'lucide-vue-next'

interface ExportOptions {
  include_pttype_groups: boolean
  icode_ids: number[] | null
}

interface ImportSummary {
  pttype_groups_added: number
  pttype_groups_skipped: number
  icode_configs_added: number
  icode_configs_updated: number
}

const icodeStore = useIcodeStore()

// ── Export state ─────────────────────────────────────────────────────────────
const includePttype = ref(true)
const icodeMode = ref<'all' | 'select'>('all')   // 'all' | 'select'
const selectedIcodeIds = ref<number[]>([])
const isExporting = ref(false)
const exportError = ref('')

const allSelected = computed(() =>
  icodeStore.items.length > 0 &&
  selectedIcodeIds.value.length === icodeStore.items.length
)

function toggleSelectAll() {
  if (allSelected.value) {
    selectedIcodeIds.value = []
  } else {
    selectedIcodeIds.value = icodeStore.items.map(i => i.id)
  }
}

function toggleIcode(id: number) {
  const idx = selectedIcodeIds.value.indexOf(id)
  if (idx >= 0) selectedIcodeIds.value.splice(idx, 1)
  else selectedIcodeIds.value.push(id)
}

function downloadJson(content: string, filename: string) {
  const blob = new Blob([content], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
}

async function handleExport() {
  exportError.value = ''
  isExporting.value = true
  try {
    const options: ExportOptions = {
      include_pttype_groups: includePttype.value,
      icode_ids: icodeMode.value === 'all' ? null : [...selectedIcodeIds.value],
    }
    const json = await invoke<string>('export_settings', { options })
    const date = new Date().toISOString().slice(0, 10)
    downloadJson(json, `setting_export_${date}.json`)
  } catch (e: unknown) {
    exportError.value = String(e)
  } finally {
    isExporting.value = false
  }
}

// ── Import state ─────────────────────────────────────────────────────────────
const fileInput = ref<HTMLInputElement | null>(null)
const selectedFile = ref<File | null>(null)
const isImporting = ref(false)
const importError = ref('')
const importResult = ref<ImportSummary | null>(null)

function onFileChange(e: Event) {
  const input = e.target as HTMLInputElement
  selectedFile.value = input.files?.[0] ?? null
  importResult.value = null
  importError.value = ''
}

async function handleImport() {
  if (!selectedFile.value) return
  importError.value = ''
  importResult.value = null
  isImporting.value = true
  try {
    const text = await selectedFile.value.text()
    const result = await invoke<ImportSummary>('import_settings', { data: text })
    importResult.value = result
    await icodeStore.loadAll()
  } catch (e: unknown) {
    importError.value = String(e)
  } finally {
    isImporting.value = false
  }
}

onMounted(() => icodeStore.loadAll())
</script>

<template>
  <div class="export-import-settings">

    <!-- ── Export ──────────────────────────────────────────────────────────── -->
    <div class="card mb-4">
      <div class="card-header">
        <Download :size="18" style="color:var(--brand-orange)" />
        ส่งออกการตั้งค่า
      </div>
      <div class="card-body">

        <!-- What to include -->
        <div class="section-label">เลือกส่วนที่ต้องการส่งออก</div>

        <label class="check-row">
          <input type="checkbox" v-model="includePttype" />
          <span>สิทธิการรักษา (pttype groups)</span>
        </label>

        <div class="divider" />

        <div class="section-label">รายการ icode</div>
        <div class="radio-row">
          <label class="radio-opt">
            <input type="radio" v-model="icodeMode" value="all" />
            <span>ทั้งหมด</span>
          </label>
          <label class="radio-opt">
            <input type="radio" v-model="icodeMode" value="select" />
            <span>เลือกเอง</span>
          </label>
        </div>

        <!-- Icode selection table -->
        <div v-if="icodeMode === 'select'" class="icode-select-box mt-3">
          <div class="icode-select-header">
            <span class="text-sm text-muted">
              เลือกแล้ว {{ selectedIcodeIds.length }} / {{ icodeStore.items.length }} รายการ
            </span>
            <button class="btn btn-ghost btn-sm" @click="toggleSelectAll">
              <component :is="allSelected ? CheckSquare : Square" :size="14" />
              {{ allSelected ? 'ยกเลิกทั้งหมด' : 'เลือกทั้งหมด' }}
            </button>
          </div>
          <div class="icode-select-list">
            <label
              v-for="item in icodeStore.items"
              :key="item.id"
              class="icode-select-row"
              :class="{ selected: selectedIcodeIds.includes(item.id) }"
            >
              <input
                type="checkbox"
                :value="item.id"
                :checked="selectedIcodeIds.includes(item.id)"
                @change="toggleIcode(item.id)"
              />
              <span class="badge badge-gray font-mono">{{ item.icode }}</span>
              <span class="icode-select-name">{{ item.service_name }}</span>
              <span v-if="item.department" class="text-muted text-sm">
                {{ item.department }}
              </span>
            </label>
            <div v-if="icodeStore.items.length === 0" class="empty-msg">
              ยังไม่มีรายการ icode
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
            {{ isExporting ? 'กำลังส่งออก...' : 'ส่งออก (.json)' }}
          </button>
        </div>

        <div v-if="exportError" class="alert alert-error mt-2">
          <AlertCircle :size="16" />{{ exportError }}
        </div>
      </div>
    </div>

    <!-- ── Import ──────────────────────────────────────────────────────────── -->
    <div class="card">
      <div class="card-header">
        <Upload :size="18" style="color:var(--brand-orange)" />
        นำเข้าการตั้งค่า
      </div>
      <div class="card-body">
        <p class="text-sm text-muted mb-3">
          นำเข้าจากไฟล์ <code>.json</code> ที่ส่งออกจากโปรแกรมนี้เท่านั้น<br/>
          icode ที่มีอยู่แล้วจะถูกอัปเดต; pttype groups ที่ซ้ำกันจะถูกข้าม
        </p>

        <!-- File picker -->
        <input
          ref="fileInput"
          type="file"
          accept=".json"
          style="display:none"
          @change="onFileChange"
        />
        <div class="file-row">
          <button class="btn btn-secondary" @click="fileInput?.click()">
            <FileJson :size="16" />
            เลือกไฟล์
          </button>
          <span v-if="selectedFile" class="file-name">
            {{ selectedFile.name }}
          </span>
          <span v-else class="text-muted text-sm">ยังไม่ได้เลือกไฟล์</span>
        </div>

        <div class="divider" />

        <div class="action-row">
          <button
            class="btn btn-primary"
            @click="handleImport"
            :disabled="!selectedFile || isImporting"
          >
            <Upload :size="16" />
            {{ isImporting ? 'กำลังนำเข้า...' : 'นำเข้า' }}
          </button>
        </div>

        <div v-if="importError" class="alert alert-error mt-2">
          <AlertCircle :size="16" />{{ importError }}
        </div>

        <div v-if="importResult" class="alert alert-success mt-2">
          <Check :size="16" />
          <div>
            <div style="font-weight:600;margin-bottom:4px">นำเข้าสำเร็จ</div>
            <div class="text-sm">
              สิทธิการรักษา:
              เพิ่มใหม่ <strong>{{ importResult.pttype_groups_added }}</strong> รายการ,
              ข้าม <strong>{{ importResult.pttype_groups_skipped }}</strong> รายการ
            </div>
            <div class="text-sm">
              icode:
              เพิ่มใหม่ <strong>{{ importResult.icode_configs_added }}</strong> รายการ,
              อัปเดต <strong>{{ importResult.icode_configs_updated }}</strong> รายการ
            </div>
          </div>
        </div>
      </div>
    </div>

  </div>
</template>

<style scoped>
.export-import-settings { display: flex; flex-direction: column; gap: 0; max-width: 700px; }
.mb-4  { margin-bottom: 16px; }
.mt-2  { margin-top: 8px; }
.mt-3  { margin-top: 12px; }
.mb-3  { margin-bottom: 12px; }
.section-label { font-size: 13px; font-weight: 600; color: var(--slate); margin-bottom: 8px; }
.check-row { display: flex; align-items: center; gap: 8px; cursor: pointer; font-size: 14px; }
.radio-row { display: flex; gap: 20px; }
.radio-opt { display: flex; align-items: center; gap: 6px; cursor: pointer; font-size: 14px; }
.action-row { display: flex; gap: 8px; }
.file-row { display: flex; align-items: center; gap: 12px; }
.file-name { font-size: 13px; font-weight: 500; color: var(--slate); }
.icode-select-box {
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  overflow: hidden;
}
.icode-select-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: 8px 12px;
  background: var(--surface-soft);
  border-bottom: 1px solid var(--border);
}
.icode-select-list { max-height: 260px; overflow-y: auto; }
.icode-select-row {
  display: flex; align-items: center; gap: 8px;
  padding: 8px 12px;
  cursor: pointer;
  font-size: 13px;
  transition: background 0.1s;
}
.icode-select-row:hover { background: var(--surface-soft); }
.icode-select-row.selected { background: #fff7ed; }
.icode-select-row input[type="checkbox"] { flex-shrink: 0; }
.icode-select-name { flex: 1; }
.empty-msg { padding: 20px; text-align: center; color: var(--muted); font-size: 13px; }
</style>
