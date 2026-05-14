<script setup lang="ts">
import { ref } from "vue";
import { X, Printer, ExternalLink } from "lucide-vue-next";

const props = withDefaults(
    defineProps<{
        visible: boolean;
        pdfUrl: string;
        title?: string;
    }>(),
    {
        title: "ตัวอย่าง PDF",
    },
);

const emit = defineEmits<{
    close: [];
}>();

const iframeRef = ref<HTMLIFrameElement | null>(null);

function closeModal() {
    emit("close");
}

function printPdf() {
    try {
        const frameWindow = iframeRef.value?.contentWindow;
        if (frameWindow) {
            frameWindow.focus();
            frameWindow.print();
            return;
        }
    } catch {
        // fallback ด้านล่าง
    }

    if (props.pdfUrl) {
        window.open(props.pdfUrl, "_blank");
    }
}

function openInNewWindow() {
    if (props.pdfUrl) {
        window.open(props.pdfUrl, "_blank");
    }
}
</script>

<template>
    <div v-if="visible" class="pdf-preview-overlay" @click.self="closeModal">
        <div class="pdf-preview-modal">
            <div class="pdf-preview-header">
                <div class="pdf-preview-title-wrap">
                    <strong>{{ props.title }}</strong>
                    <span class="pdf-preview-hint">
                        แสดงไฟล์ PDF โดยยังไม่บันทึกลงเครื่อง
                    </span>
                </div>
                <div class="pdf-preview-actions">
                    <button
                        class="btn btn-secondary btn-sm"
                        @click="printPdf"
                        :disabled="!pdfUrl"
                    >
                        <Printer :size="14" />
                        พิมพ์
                    </button>
                    <button
                        class="btn btn-ghost btn-sm"
                        @click="openInNewWindow"
                        :disabled="!pdfUrl"
                    >
                        <ExternalLink :size="14" />
                        เปิดแยกหน้าต่าง
                    </button>
                    <button class="btn btn-ghost btn-sm" @click="closeModal">
                        <X :size="14" />
                    </button>
                </div>
            </div>

            <div class="pdf-preview-body">
                <iframe
                    v-if="pdfUrl"
                    ref="iframeRef"
                    :src="`${pdfUrl}#toolbar=1&navpanes=0&scrollbar=1`"
                    title="PDF Preview"
                    class="pdf-frame"
                ></iframe>
                <div v-else class="pdf-preview-empty">
                    ไม่พบไฟล์ PDF สำหรับแสดงตัวอย่าง
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
.pdf-preview-overlay {
    position: fixed;
    inset: 0;
    z-index: 4000;
    background: rgba(15, 23, 42, 0.62);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 20px;
}

.pdf-preview-modal {
    width: min(1100px, 96vw);
    height: min(900px, 92vh);
    background: white;
    border-radius: 12px;
    box-shadow: 0 20px 60px rgba(15, 23, 42, 0.35);
    overflow: hidden;
    display: flex;
    flex-direction: column;
}

.pdf-preview-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 14px 16px;
    border-bottom: 1px solid var(--hairline);
    background: #f8fafc;
}

.pdf-preview-title-wrap {
    display: flex;
    flex-direction: column;
    gap: 2px;
}

.pdf-preview-title-wrap strong {
    color: var(--charcoal);
    font-size: 15px;
}

.pdf-preview-hint {
    color: var(--slate);
    font-size: 12px;
}

.pdf-preview-actions {
    display: flex;
    align-items: center;
    gap: 8px;
}

.pdf-preview-body {
    flex: 1;
    min-height: 0;
    background: #e2e8f0;
}

.pdf-frame {
    width: 100%;
    height: 100%;
    border: 0;
    background: white;
}

.pdf-preview-empty {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--slate);
    font-size: 14px;
}
</style>
