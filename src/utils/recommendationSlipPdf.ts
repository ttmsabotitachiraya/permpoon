import html2canvas from "html2canvas";
import { jsPDF } from "jspdf";

export interface RecommendationSlipPdfInput {
  patient: {
    hn: string;
    fname: string;
    lname: string;
  };
  items: Array<{
    service_name: string;
    department?: string | null;
  }>;
  processDateText: string;
  pttypeText: string;
}

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function formatThaiFullDate(value: string): string {
  if (!value || value === "-") return "-";

  const isoMatch = value.match(/^(\d{4})-(\d{2})-(\d{2})$/);
  const thaiShortMatch = value.match(/^(\d{2})\/(\d{2})\/(\d{4})$/);

  let date: Date | null = null;

  if (isoMatch) {
    const [, year, month, day] = isoMatch;
    date = new Date(Number(year), Number(month) - 1, Number(day));
  } else if (thaiShortMatch) {
    const [, day, month, buddhistYear] = thaiShortMatch;
    date = new Date(Number(buddhistYear) - 543, Number(month) - 1, Number(day));
  } else {
    const parsed = new Date(value);
    if (!Number.isNaN(parsed.getTime())) {
      date = parsed;
    }
  }

  if (!date || Number.isNaN(date.getTime())) return value;

  const day = String(date.getDate()).padStart(2, "0");
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const year = String(date.getFullYear() + 543);

  return `${day}/${month}/${year}`;
}

function buildSlipHtml(input: RecommendationSlipPdfInput): string {
  const rows = input.items
    .map(
      (item, index) => `
                <tr>
                    <td class="td-num">${index + 1}</td>
                    <td class="td-service">${escapeHtml(item.service_name)}</td>
                    <td class="td-dept">${escapeHtml(item.department || "-")}</td>
                    <td class="td-note"></td>
                </tr>`,
    )
    .join("");

  return `
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
                            <span class="slip-val mono">${escapeHtml(input.patient.hn)}</span>
                        </div>
                        <div class="slip-field half">
                            <span class="slip-lbl">วันที่</span>
                            <span class="slip-val">${escapeHtml(formatThaiFullDate(input.processDateText))}</span>
                        </div>
                    </div>
                    <div class="slip-field">
                        <span class="slip-lbl">ชื่อ-นามสกุล</span>
                        <span class="slip-val">${escapeHtml(`${input.patient.fname} ${input.patient.lname}`)}</span>
                    </div>
                    <div class="slip-field no-margin-bottom">
                        <span class="slip-lbl">สิทธิการรักษา</span>
                        <span class="slip-val">${escapeHtml(input.pttypeText || "-")}</span>
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
                    <tbody>${rows}</tbody>
                </table>
            </div>

            <div class="slip-footer">
                <div class="slip-footer-line"></div>
                <p>กรุณานำใบนี้ไปรับบริการที่จุดให้บริการ</p>
            </div>
        </div>`;
}

const slipCss = `
    .slip-page {
        width: 650px;
        padding: 48px 48px 36px 64px;
        background: #ffffff;
        color: #1e293b;
        font-family: Thonburi, "Noto Sans Thai", "Sarabun", Arial, sans-serif;
        box-sizing: border-box;
        font-size: 18px;
        line-height: 1.35;
    }

    *, *::before, *::after {
        box-sizing: border-box;
    }

    .slip-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 12px;
        gap: 14px;
    }

    .slip-logo-wrap {
        display: flex;
        align-items: center;
        gap: 12px;
    }

    .slip-brand {
        display: flex;
        flex-direction: column;
        line-height: 1.15;
    }

    .slip-brand-name {
        font-size: 34px;
        font-weight: 700;
        color: #ea580c;
        letter-spacing: -0.3px;
    }

    .slip-brand-tagline {
        font-size: 16px;
        color: #94a3b8;
        margin-top: 4px;
    }

    .slip-title-wrap {
        text-align: right;
        flex-shrink: 0;
    }

    .slip-title {
        display: inline-block;
        font-size: 20px; /* reduced from 26px */
        font-weight: 700;
        color: #ea580c;
        background: #fff7ed;
        border: 2px solid #f97316;
        border-radius: 10px;
        padding: 8px 12px; /* slightly reduced to match smaller text */
        white-space: nowrap;
    }

    .slip-divider {
        height: 6px;
        background: linear-gradient(90deg, #f97316 0%, #fbbf24 60%, #fff7ed 100%);
        border-radius: 999px;
        margin: 12px 0 18px;
    }

    .slip-section {
        margin-bottom: 20px;
    }

    .slip-section-label {
        font-size: 20px;
        font-weight: 700;
        color: #ea580c;
        letter-spacing: 0.3px;
        border-left: 5px solid #f97316;
        padding-left: 10px;
        margin-bottom: 10px;
    }

    .slip-info-box {
        border: 1px solid #e2e8f0;
        border-radius: 12px;
        padding: 16px 18px;
        background: #fafbfc;
    }

    .slip-row {
        display: flex;
        gap: 18px;
        margin-bottom: 10px;
    }

    .slip-field {
        display: flex;
        align-items: baseline;
        gap: 8px;
        margin-bottom: 10px;
        min-width: 0;
    }

    .slip-field.half {
        flex: 1;
        margin-bottom: 0;
    }

    .no-margin-bottom {
        margin-bottom: 0;
    }

    .slip-lbl {
        font-size: 18px;
        font-weight: 700;
        color: #475569;
        white-space: nowrap;
    }

    .slip-lbl::after {
        content: ':';
    }

    .slip-val {
        font-size: 20px;
        font-weight: 600;
        color: #0f172a;
        word-break: break-word;
    }

    .mono {
        font-family: "Courier New", monospace;
        font-weight: 800;
        color: #ea580c;
        font-size: 22px;
    }

    .slip-table {
        width: 100%;
        border-collapse: collapse;
        font-size: 14px; /* reduced font size for the services table */
        table-layout: fixed;
    }

    .slip-table thead tr {
        background: #fff7ed;
    }

    .slip-table th {
        padding: 10px 12px;
        font-size: 14px; /* reduced header font size */
        font-weight: 700;
        color: #c2410c;
        text-align: left;
        border: 1px solid #fed7aa;
    }

    .slip-table td {
        padding: 10px 12px;
        border: 1px solid #e2e8f0;
        color: #1e293b;
        vertical-align: top;
        word-break: break-word;
    }

    .slip-table tbody tr:nth-child(even) td {
        background: #fff7ed;
    }

    .th-num, .td-num {
        width: 60px; /* restored to original */
        text-align: center;
    }

    .th-dept, .td-dept {
        width: 100px; /* reduced from 120px */
    }

    .th-note, .td-note {
        width: 110px;
    }

    .slip-footer {
        margin-top: 20px;
        text-align: center;
    }

    .slip-footer-line {
        height: 1px;
        background: #e2e8f0;
        margin-bottom: 10px;
    }

    .slip-footer p {
        margin: 0;
        font-size: 14px;
        color: #94a3b8;
    }
`;

async function buildRecommendationSlipPdfBlob(
  input: RecommendationSlipPdfInput,
): Promise<Blob> {
  if (input.items.length === 0) {
    throw new Error("ไม่มีรายการบริการสำหรับสร้าง PDF");
  }

  const renderHost = document.createElement("div");
  renderHost.setAttribute("data-slip-pdf-render", "true");
  renderHost.style.position = "fixed";
  renderHost.style.left = "-20000px";
  renderHost.style.top = "0";
  renderHost.style.pointerEvents = "none";
  renderHost.style.opacity = "0";
  renderHost.innerHTML = `<style>${slipCss}</style>${buildSlipHtml(input)}`;
  document.body.appendChild(renderHost);

  try {
    await new Promise<void>((resolve) =>
      requestAnimationFrame(() => resolve()),
    );
    await new Promise<void>((resolve) => setTimeout(resolve, 120));

    const pageEl = renderHost.querySelector(".slip-page") as HTMLElement | null;
    if (!pageEl) {
      throw new Error("ไม่สามารถสร้างเนื้อหาใบแนะนำบริการได้");
    }

    const canvas = await html2canvas(pageEl, {
      scale: 2,
      backgroundColor: "#ffffff",
      useCORS: true,
      logging: false,
    });

    const pdf = new jsPDF({
      orientation: "portrait",
      unit: "mm",
      format: "a5",
      compress: true,
    });

    const pageWidth = pdf.internal.pageSize.getWidth();
    const pageHeight = pdf.internal.pageSize.getHeight();
    const margin = 6;
    const maxImageWidth = pageWidth - margin * 2;
    const maxImageHeight = pageHeight - margin * 2;
    const widthRatio = maxImageWidth / canvas.width;
    const heightRatio = maxImageHeight / canvas.height;
    const scaleRatio = Math.min(widthRatio, heightRatio);
    const imageWidth = canvas.width * scaleRatio;
    const imageHeight = canvas.height * scaleRatio;
    const x = (pageWidth - imageWidth) / 2;
    const y = margin;
    const imageData = canvas.toDataURL("image/png");

    pdf.addImage(
      imageData,
      "PNG",
      x,
      y,
      imageWidth,
      imageHeight,
      undefined,
      "FAST",
    );

    return pdf.output("blob");
  } finally {
    renderHost.remove();
  }
}

export async function createRecommendationSlipPdfUrl(
  input: RecommendationSlipPdfInput,
): Promise<string> {
  const pdfBlob = await buildRecommendationSlipPdfBlob(input);
  return URL.createObjectURL(pdfBlob);
}

export async function createRecommendationSlipPdfBase64(
  input: RecommendationSlipPdfInput,
): Promise<string> {
  const pdfBlob = await buildRecommendationSlipPdfBlob(input);
  const arrayBuffer = await pdfBlob.arrayBuffer();
  const bytes = new Uint8Array(arrayBuffer);

  let binary = "";
  const chunkSize = 0x8000;
  for (let i = 0; i < bytes.length; i += chunkSize) {
    const chunk = bytes.subarray(i, i + chunkSize);
    binary += String.fromCharCode(...chunk);
  }

  return btoa(binary);
}
