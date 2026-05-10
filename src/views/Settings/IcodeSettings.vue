<script setup lang="ts">
import {
    ref,
    onMounted,
    watch,
    defineComponent,
    onBeforeUnmount,
    nextTick,
    h,
    computed,
} from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useIcodeStore } from "../../stores/icodeStore";
import { usePttypeStore } from "../../stores/pttypeStore";
import { useConnectionStore } from "../../stores/connectionStore";
import { Search, Trash2, Tag, AlertCircle, Edit, X } from "lucide-vue-next";
import type { IcodeConfig } from "../../types/icode";

const icodeStore = useIcodeStore();
const pttypeStore = usePttypeStore();
const connStore = useConnectionStore();

const icodeInput = ref("");
const serviceName = ref("");
const isFetching = ref(false);
const fetchError = ref("");

// form
const form = ref<Omit<IcodeConfig, "id">>({
    icode: "",
    service_name: "",
    is_enabled: true,
    age_min: null,
    age_max: null,
    gender_restrict: null,
    freq_type: null,
    freq_value: null,
    pttype_group_ids: [],
    department: null,
});

// Computed string-backed inputs for age fields so we can allow empty value
// while keeping the underlying type as number | null in `form`.
const ageMin = computed<string>({
    get() {
        const v = form.value.age_min;
        return v === null || v === undefined ? "" : String(v);
    },
    set(val: string) {
        const digits = (val || "").replace(/\D/g, "");
        form.value.age_min = digits === "" ? null : parseInt(digits, 10);
    },
});

const ageMax = computed<string>({
    get() {
        const v = form.value.age_max;
        return v === null || v === undefined ? "" : String(v);
    },
    set(val: string) {
        const digits = (val || "").replace(/\D/g, "");
        form.value.age_max = digits === "" ? null : parseInt(digits, 10);
    },
});
const editId = ref<number | null>(null);
const saveMsg = ref("");
const saveError = ref("");
const showModal = ref(false);
const confirmDeleteId = ref<number | null>(null);
// prevent body scroll when modal is open
watch(showModal, (val) => {
    try {
        document.body.style.overflow = val ? "hidden" : "";
        if (val) saveError.value = ""; // clear modal errors when opened
    } catch (e) {
        // noop in non-browser env
    }
});

onMounted(async () => {
    await icodeStore.loadAll();
    await pttypeStore.loadAll();
});

async function searchIcode() {
    if (!icodeInput.value.trim()) return;
    if (!connStore.isConnected) {
        fetchError.value = "ยังไม่ได้เชื่อมต่อ";
        return;
    }
    isFetching.value = true;
    fetchError.value = "";
    serviceName.value = "";
    try {
        const name = await invoke<string | null>("get_icode_name", {
            config: connStore.config,
            icode: icodeInput.value.trim(),
        });
        if (name) {
            serviceName.value = name;
            form.value.icode = icodeInput.value.trim();
            form.value.service_name = name;
            // เปิด modal ตั้งค่าเงื่อนไขทันทีหลังค้นหาเจอ
            showModal.value = true;
        } else {
            fetchError.value = "ไม่พบ icode นี้ในฐานข้อมูล HOSxP";
        }
    } catch (e: any) {
        fetchError.value = String(e);
    } finally {
        isFetching.value = false;
    }
}

function resetForm() {
    form.value = {
        icode: "",
        service_name: "",
        is_enabled: true,
        age_min: null,
        age_max: null,
        gender_restrict: null,
        freq_type: null,
        freq_value: null,
        pttype_group_ids: [],
        department: null,
    };
    icodeInput.value = "";
    serviceName.value = "";
    editId.value = null;
    fetchError.value = "";
    saveMsg.value = "";
    saveError.value = "";
}

function editItem(item: IcodeConfig) {
    editId.value = item.id;
    icodeInput.value = item.icode;
    serviceName.value = item.service_name;
    form.value = { ...item };
    // เปิด modal เพื่อแก้เงื่อนไข
    showModal.value = true;
}

async function handleSave() {
    // required validation
    if (!form.value.icode || !form.value.service_name) {
        saveError.value = "กรุณากรอก icode และชื่อบริการ";
        return;
    }
    if (!form.value.department || (form.value.department || "").trim() === "") {
        saveError.value = "กรุณาระบุแผนกที่ให้บริการ";
        return;
    }
    if (
        !form.value.pttype_group_ids ||
        form.value.pttype_group_ids.length === 0
    ) {
        saveError.value = "กรุณาเลือกอย่างน้อย 1 สิทธิ์ที่แนะนำ";
        return;
    }

    saveError.value = "";
    try {
        await icodeStore.saveIcode({
            ...form.value,
            ...(editId.value ? { id: editId.value } : {}),
        } as any);
        saveMsg.value = `บันทึก icode "${form.value.icode}" สำเร็จ`;
        // ปิด modal อัตโนมัติหลังบันทึก
        showModal.value = false;
        resetForm();
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
        await icodeStore.removeIcode(id);
    } catch (e: unknown) {
        saveError.value = `ลบไม่สำเร็จ: ${String(e)}`;
    }
}

function togglePttype(id: number) {
    const idx = form.value.pttype_group_ids.indexOf(id);
    if (idx >= 0) form.value.pttype_group_ids.splice(idx, 1);
    else form.value.pttype_group_ids.push(id);
}

function setGenderRestrict(value: IcodeConfig["gender_restrict"]) {
    form.value.gender_restrict = value;
}

function getPttypeAliases(ids: number[]) {
    return ids
        .map((id) => pttypeStore.groups.find((g) => g.id === id)?.alias || id)
        .join(", ");
}

// Local component: TruncatedText
const TruncatedText = defineComponent({
    name: "TruncatedText",
    props: {
        text: { type: String, required: true },
    },
    setup(props) {
        const elRef = ref<HTMLElement | null>(null);
        let ro: ResizeObserver | null = null;

        const update = () => {
            const el = elRef.value;
            if (!el) return;
            const text = props.text ?? el.textContent ?? "";
            const isTruncated =
                el.scrollWidth > el.clientWidth ||
                el.scrollHeight > el.clientHeight;
            if (isTruncated) el.setAttribute("title", String(text));
            else el.removeAttribute("title");
        };

        onMounted(() => {
            nextTick(() => update());
            if (typeof ResizeObserver !== "undefined") {
                ro = new ResizeObserver(update);
                if (elRef.value) ro.observe(elRef.value);
            }
            window.addEventListener("resize", update);
        });

        onBeforeUnmount(() => {
            if (ro) ro.disconnect();
            window.removeEventListener("resize", update);
        });

        watch(
            () => props.text,
            () => nextTick(update),
        );

        return () =>
            h("span", { ref: elRef, class: "text-sm pttype-text" }, props.text);
    },
});
</script>

<template>
    <div class="icode-settings">
        <!-- Form Card -->
        <div class="card mb-4">
            <div class="card-header" style="justify-content: space-between">
                <div style="display: flex; align-items: center; gap: 8px">
                    <Tag :size="18" style="color: var(--brand-orange)" />
                    {{
                        editId
                            ? `แก้ไข icode: ${form.icode}`
                            : "เพิ่ม icode ใหม่"
                    }}
                </div>
                <button
                    v-if="editId"
                    class="btn btn-ghost btn-sm"
                    @click="resetForm"
                >
                    <X :size="14" />ยกเลิก
                </button>
            </div>
            <div class="card-body">
                <!-- icode search -->
                <div class="icode-search-row">
                    <div style="flex: 1">
                        <label class="form-label">รหัส icode</label>
                        <div class="search-input-row">
                            <input
                                v-model="icodeInput"
                                class="form-input font-mono"
                                placeholder="เช่น 3003038"
                                @keydown.enter="searchIcode"
                            />
                            <button
                                class="btn btn-secondary"
                                @click="searchIcode"
                                :disabled="isFetching"
                            >
                                <Search :size="16" />
                                {{ isFetching ? "..." : "ค้นหา" }}
                            </button>
                        </div>
                    </div>
                    <div style="flex: 2">
                        <label class="form-label">ชื่อบริการ</label>
                        <input
                            v-model="form.service_name"
                            class="form-input"
                            :placeholder="serviceName || 'ค้นหา icode ก่อน'"
                        />
                    </div>
                </div>
                <div v-if="fetchError" class="alert alert-error mt-2">
                    <AlertCircle :size="16" />{{ fetchError }}
                </div>

                <!-- Modal -->
                <div
                    v-if="showModal"
                    class="modal-overlay"
                    @click.self="showModal = false"
                >
                    <div class="modal">
                        <div class="modal-header">
                            <div
                                style="
                                    display: flex;
                                    align-items: center;
                                    gap: 8px;
                                "
                            >
                                <Tag
                                    :size="16"
                                    style="
                                        color: var(--brand-orange);
                                        margin-right: 8px;
                                    "
                                />
                                <div class="modal-title">
                                    <span class="modal-title-label"
                                        >ตั้งค่าเงื่อนไขสำหรับ:</span
                                    >
                                    <span class="modal-title-subject">{{
                                        form.service_name || form.icode
                                    }}</span>
                                </div>
                            </div>
                            <button
                                class="btn btn-ghost btn-sm"
                                @click="showModal = false"
                            >
                                <X :size="14" />
                            </button>
                        </div>
                        <div class="modal-body">
                            <div
                                v-if="saveError"
                                class="alert alert-error mt-2"
                            >
                                {{ saveError }}
                            </div>
                            <!-- Department -->
                            <div
                                style="margin-bottom: 12px"
                                class="cond-section"
                            >
                                <label class="form-label"
                                    >แผนกที่ให้บริการ
                                    <span class="required">*</span></label
                                >
                                <input
                                    v-model="form.department"
                                    class="form-input"
                                    placeholder="เช่น งานเวชกรรม (OPD)"
                                />
                                <div
                                    class="text-sm text-muted"
                                    style="margin-top: 6px"
                                >
                                    ชื่อแผนกที่จะแสดงในใบสั่ง/เอกสารสำหรับผู้ป่วย
                                </div>
                            </div>
                            <div class="conditions-grid modal-vertical">
                                <!-- Age -->
                                <div class="cond-section">
                                    <div class="cond-label">เงื่อนไขอายุ</div>
                                    <div class="age-row">
                                        <input
                                            v-model="ageMin"
                                            type="text"
                                            inputmode="numeric"
                                            class="form-input"
                                            placeholder="ขั้นต่ำ"
                                            style="width: 100px"
                                            maxlength="3"
                                        />
                                        <span class="text-muted">–</span>
                                        <input
                                            v-model="ageMax"
                                            type="text"
                                            inputmode="numeric"
                                            class="form-input"
                                            placeholder="สูงสุด"
                                            style="width: 100px"
                                            maxlength="3"
                                        />
                                        <span class="text-muted text-sm"
                                            >ปี (ว่าง = ไม่จำกัด)</span
                                        >
                                    </div>
                                </div>

                                <!-- Gender -->
                                <div class="cond-section">
                                    <div class="cond-label">เงื่อนไขเพศ</div>
                                    <div class="selection-options">
                                        <label
                                            class="selection-option"
                                            :class="{
                                                selected:
                                                    form.gender_restrict ===
                                                    null,
                                            }"
                                        >
                                            <input
                                                type="checkbox"
                                                class="selection-option-input"
                                                :checked="
                                                    form.gender_restrict ===
                                                    null
                                                "
                                                @change="
                                                    setGenderRestrict(null)
                                                "
                                            />
                                            <span class="selection-option-text"
                                                >ทุกเพศ</span
                                            >
                                        </label>
                                        <label
                                            class="selection-option"
                                            :class="{
                                                selected:
                                                    form.gender_restrict ===
                                                    'M',
                                            }"
                                        >
                                            <input
                                                type="checkbox"
                                                class="selection-option-input"
                                                :checked="
                                                    form.gender_restrict === 'M'
                                                "
                                                @change="setGenderRestrict('M')"
                                            />
                                            <span class="selection-option-text"
                                                >ชาย</span
                                            >
                                        </label>
                                        <label
                                            class="selection-option"
                                            :class="{
                                                selected:
                                                    form.gender_restrict ===
                                                    'F',
                                            }"
                                        >
                                            <input
                                                type="checkbox"
                                                class="selection-option-input"
                                                :checked="
                                                    form.gender_restrict === 'F'
                                                "
                                                @change="setGenderRestrict('F')"
                                            />
                                            <span class="selection-option-text"
                                                >หญิง</span
                                            >
                                        </label>
                                    </div>
                                </div>

                                <!-- Frequency -->
                                <div class="cond-full cond-section">
                                    <div class="cond-label">
                                        เงื่อนไขความถี่
                                    </div>
                                    <div class="freq-row">
                                        <select
                                            v-model="form.freq_type"
                                            class="form-input freq-type-select"
                                        >
                                            <option :value="null">
                                                ไม่จำกัด
                                            </option>
                                            <option value="days">
                                                ห่างอย่างน้อย N วัน
                                            </option>
                                            <option value="months">
                                                ห่างอย่างน้อย N เดือน
                                            </option>
                                            <option value="years">
                                                ทุก N ปี
                                            </option>
                                            <option value="per_week">
                                                ไม่เกิน N ครั้ง/สัปดาห์
                                            </option>
                                            <option value="per_year">
                                                ไม่เกิน N ครั้ง/ปี
                                            </option>
                                            <option value="total_limit">
                                                รวมทั้งหมดไม่เกิน N ครั้ง
                                            </option>
                                        </select>
                                        <input
                                            v-if="form.freq_type"
                                            v-model.number="form.freq_value"
                                            type="number"
                                            class="form-input freq-value-input"
                                            placeholder="N"
                                        />
                                    </div>
                                </div>

                                <!-- Pttype -->
                                <div class="cond-full cond-section">
                                    <div class="cond-label">
                                        แนะนำสำหรับสิทธิ์
                                        <span class="required">*</span>
                                    </div>
                                    <div
                                        v-if="pttypeStore.groups.length === 0"
                                        class="alert alert-warning"
                                        style="display: inline-flex"
                                    >
                                        <AlertCircle :size="16" />
                                        กรุณาเพิ่มสิทธิ์ที่ "ตั้งค่า
                                        สิทธิการรักษา" ก่อน
                                    </div>
                                    <div v-else class="selection-options">
                                        <label
                                            v-for="g in pttypeStore.groups"
                                            :key="g.id"
                                            class="selection-option"
                                            :class="{
                                                selected:
                                                    form.pttype_group_ids.includes(
                                                        g.id,
                                                    ),
                                            }"
                                        >
                                            <input
                                                type="checkbox"
                                                class="selection-option-input"
                                                :checked="
                                                    form.pttype_group_ids.includes(
                                                        g.id,
                                                    )
                                                "
                                                @change="togglePttype(g.id)"
                                            />
                                            <span
                                                class="selection-option-text"
                                                >{{ g.alias }}</span
                                            >
                                        </label>
                                    </div>
                                </div>

                                <!-- Status -->
                                <div class="cond-section">
                                    <div class="cond-label">สถานะ</div>
                                    <div class="toggle-row">
                                        <button
                                            class="toggle"
                                            :class="{ active: form.is_enabled }"
                                            @click="
                                                form.is_enabled =
                                                    !form.is_enabled
                                            "
                                        ></button>
                                        <span class="text-sm">{{
                                            form.is_enabled
                                                ? "เปิดใช้งาน"
                                                : "ปิดใช้งาน"
                                        }}</span>
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div class="modal-footer">
                            <div
                                style="
                                    display: flex;
                                    gap: 8px;
                                    justify-content: flex-end;
                                "
                            >
                                <button
                                    class="btn btn-primary"
                                    @click="handleSave"
                                    :disabled="
                                        !form.department ||
                                        !(
                                            form.pttype_group_ids &&
                                            form.pttype_group_ids.length
                                        ) ||
                                        !form.icode ||
                                        !form.service_name
                                    "
                                >
                                    บันทึก
                                </button>
                                <button
                                    class="btn btn-ghost"
                                    @click="showModal = false"
                                >
                                    ปิด
                                </button>
                            </div>
                        </div>
                    </div>
                </div>

                <div v-if="saveMsg" class="alert alert-success mt-2">
                    {{ saveMsg }}
                </div>
                <div v-if="saveError" class="alert alert-error mt-2">
                    <AlertCircle :size="16" />{{ saveError }}
                </div>
            </div>
        </div>

        <!-- List -->
        <div class="card">
            <div class="card-header">
                <Tag :size="18" style="color: var(--brand-orange)" />
                icode ที่ตั้งค่าแล้ว
                <span class="badge badge-orange" style="margin-left: auto"
                    >{{ icodeStore.items.length }} รายการ</span
                >
            </div>
            <div class="card-body" style="padding: 0">
                <div v-if="icodeStore.items.length === 0" class="empty-state">
                    <Tag
                        :size="32"
                        style="color: var(--muted); margin-bottom: 8px"
                    />
                    <p>ยังไม่มี icode ที่ตั้งค่า</p>
                </div>
                <table v-else class="table icode-table" style="font-size: 13px">
                    <thead>
                        <tr>
                            <th>ICODE</th>
                            <th>ชื่อบริการ</th>
                            <th>สิทธิ์</th>
                            <th>แผนก</th>
                            <th>สถานะ</th>
                            <th></th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr v-for="item in icodeStore.items" :key="item.id">
                            <td>
                                <span class="badge badge-gray font-mono">{{
                                    item.icode
                                }}</span>
                            </td>
                            <td class="service-cell">
                                <span
                                    class="service-name-text"
                                    :title="item.service_name"
                                    >{{ item.service_name }}</span
                                >
                            </td>
                            <td>
                                <TruncatedText
                                    v-if="item.pttype_group_ids.length"
                                    :text="
                                        getPttypeAliases(item.pttype_group_ids)
                                    "
                                />
                                <span v-else class="text-muted text-sm"
                                    >- ไม่ระบุ -</span
                                >
                            </td>
                            <td class="text-sm">
                                {{ item.department || "- ไม่ระบุ -" }}
                            </td>
                            <td>
                                <button
                                    class="toggle"
                                    :class="{ active: item.is_enabled }"
                                    @click="
                                        icodeStore.toggleEnabled(
                                            item.id,
                                            !item.is_enabled,
                                        )
                                    "
                                ></button>
                            </td>
                            <td style="text-align: right">
                                <div
                                    style="
                                        display: flex;
                                        gap: 4px;
                                        justify-content: flex-end;
                                    "
                                >
                                    <button
                                        class="btn btn-secondary btn-sm"
                                        @click="editItem(item)"
                                    >
                                        <Edit :size="13" />
                                    </button>
                                    <button
                                        class="btn btn-danger btn-sm"
                                        @click="requestRemove(item.id)"
                                    >
                                        <Trash2 :size="13" />
                                    </button>
                                </div>
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
            @click.self="cancelRemove"
        >
            <div class="delete-dialog">
                <p class="delete-dialog-title">ยืนยันการลบ?</p>
                <p class="delete-dialog-sub">
                    ต้องการลบ "{{
                        icodeStore.items.find((i) => i.id === confirmDeleteId)
                            ?.service_name
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
.icode-settings {
    display: flex;
    flex-direction: column;
    max-width: 900px;
}
.mb-4 {
    margin-bottom: 16px;
}
.mt-2 {
    margin-top: 8px;
}
.icode-search-row {
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
.conditions-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 20px;
}
.cond-full {
    grid-column: 1 / -1;
}
.cond-label {
    font-size: 13px;
    font-weight: 600;
    color: var(--slate);
    margin-bottom: 8px;
}
.age-row,
.freq-row {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
}
.freq-type-select {
    flex: 1 1 320px;
    min-width: 260px;
}
.freq-value-input {
    width: 96px;
    text-align: center;
}
.selection-options {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
}
.selection-option {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border: 1px solid var(--hairline);
    border-radius: var(--radius-md);
    background: var(--canvas);
    cursor: pointer;
    transition: all 0.15s ease;
}
.selection-option:hover {
    border-color: var(--brand-orange-light);
    background: var(--surface-soft);
}
.selection-option.selected {
    border-color: var(--brand-orange);
    background: var(--surface-soft);
}
.selection-option-input {
    accent-color: var(--brand-orange);
}
.selection-option-text {
    font-size: 14px;
    font-weight: 400;
    color: inherit;
}
.toggle-row {
    display: flex;
    align-items: center;
    gap: 10px;
}
.action-row {
    display: flex;
    gap: 8px;
}
.empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 40px;
    color: var(--slate);
}
.icode-table {
    table-layout: fixed;
}
.icode-table th:nth-child(1) {
    width: 120px;
}
.icode-table th:nth-child(3) {
    width: 220px;
}
.icode-table th:nth-child(4) {
    width: 180px;
}
.icode-table th:nth-child(5) {
    width: 100px;
}
.icode-table th:nth-child(6) {
    width: 96px;
}
.service-cell {
    max-width: 0;
}
.service-name-text,
.pttype-text {
    display: block;
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

/* Modal styles */
.modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.45); /* a bit darker */
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 3000; /* make sure it's above app chrome */
    padding: 20px;
}
.modal {
    position: relative;
    background: rgba(
        255,
        255,
        255,
        0.98
    ); /* nearly opaque white to avoid input ghosting */
    border-radius: 8px;
    width: 960px;
    max-width: 96vw;
    /* make modal fit viewport height so internal scrolling is not needed */
    height: calc(100vh - 48px);
    overflow: visible; /* remove internal scrollbar */
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.25);
    border: 1px solid rgba(0, 0, 0, 0.08);
    z-index: 3001;
}
.modal-header,
.modal-footer {
    padding: 12px 16px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-bottom: 1px solid rgba(0, 0, 0, 0.04);
}
.modal-body {
    padding: 16px;
}

/* Ensure inputs inside modal render normally */
.modal .form-input {
    background: #fff;
    box-shadow: none;
}

/* Make conditions stack vertically inside modal */
.modal .conditions-grid.modal-vertical {
    display: grid;
    grid-template-columns: 1fr;
    gap: 18px;
}
.modal .cond-section {
    padding: 8px 0;
    border-bottom: 1px solid rgba(0, 0, 0, 0.04);
}
.modal .cond-section:last-child {
    border-bottom: none;
}
.cond-label {
    font-size: 14px;
    font-weight: 800;
    color: var(--slate);
    margin-bottom: 8px;
    display: flex;
    align-items: center;
}
.form-label {
    font-size: 14px;
    font-weight: 800;
    color: var(--slate);
    display: flex;
    align-items: center;
    gap: 8px;
}
.required {
    color: var(--danger, #d23);
    margin-left: 6px;
    font-weight: 700;
}

.modal-title {
    display: flex;
    align-items: baseline;
    gap: 6px;
    flex-wrap: wrap;
    font-size: 16px;
    color: var(--slate);
}
.modal-title-label {
    font-weight: 900;
}
.modal-title-subject {
    font-weight: 400;
    color: var(--charcoal);
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
