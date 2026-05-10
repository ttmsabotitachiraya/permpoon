<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { RouterView, RouterLink, useRoute } from "vue-router";
import { useConnectionStore } from "./stores/connectionStore";
import {
    Lightbulb,
    Settings,
    Wifi,
    WifiOff,
    Shield,
    Tag,
    ChevronDown,
    Building2,
    ArrowLeftRight,
} from "lucide-vue-next";

const route = useRoute();
const connStore = useConnectionStore();
const settingsOpen = ref(false);

function toggleSettings() {
    settingsOpen.value = !settingsOpen.value;
}

function closeSettings() {
    settingsOpen.value = false;
}

// ปิด dropdown เมื่อคลิกที่อื่น
function onWindowClick(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (!target.closest(".settings-dropdown-wrapper")) {
        settingsOpen.value = false;
    }
}

watch(
    () => route.path,
    () => {
        settingsOpen.value = false;
    },
);

onMounted(async () => {
    await connStore.loadConfig();
    await connStore.testConnection();
    window.addEventListener("click", onWindowClick);
});
</script>

<template>
    <div class="app-shell">
        <!-- ───────────── TOP MENU BAR ───────────── -->
        <header class="topbar">
            <!-- Logo -->
            <RouterLink to="/" class="topbar-logo" @click="closeSettings">
                <div class="logo-icon">
                    <svg
                        width="36"
                        height="36"
                        viewBox="0 0 100 100"
                        xmlns="http://www.w3.org/2000/svg"
                        aria-hidden="true"
                    >
                        <defs>
                            <linearGradient
                                id="lg1"
                                x1="0%"
                                y1="0%"
                                x2="0%"
                                y2="100%"
                            >
                                <stop
                                    offset="0%"
                                    style="stop-color: #f97316; stop-opacity: 1"
                                />
                                <stop
                                    offset="100%"
                                    style="stop-color: #fdba74; stop-opacity: 1"
                                />
                            </linearGradient>
                            <linearGradient
                                id="lg2"
                                x1="0%"
                                y1="0%"
                                x2="0%"
                                y2="100%"
                            >
                                <stop
                                    offset="0%"
                                    style="stop-color: #fbbf24; stop-opacity: 1"
                                />
                                <stop
                                    offset="100%"
                                    style="stop-color: #fff7ed; stop-opacity: 1"
                                />
                            </linearGradient>
                        </defs>
                        <path
                            d="M5 55 L50 72 L95 55 L50 38 Z"
                            fill="url(#lg1)"
                        />
                        <path
                            d="M5 55 L5 70 L50 87 L50 72 Z"
                            fill="url(#lg1)"
                        />
                        <path
                            d="M95 55 L95 70 L50 87 L50 72 Z"
                            fill="url(#lg1)"
                        />
                        <path
                            d="M5 20 L50 37 L95 20 L50 3 Z"
                            fill="url(#lg2)"
                        />
                        <path
                            d="M5 20 L5 35 L50 52 L50 37 Z"
                            fill="url(#lg2)"
                        />
                        <path
                            d="M95 20 L95 35 L50 52 L50 37 Z"
                            fill="url(#lg2)"
                        />
                        <text
                            x="50"
                            y="98"
                            font-family="Arial, sans-serif"
                            font-size="13"
                            text-anchor="middle"
                            fill="#ea580c"
                            font-weight="900"
                            letter-spacing="2"
                        >
                            P O O N
                        </text>
                    </svg>
                </div>
                <div class="logo-text">
                    <span class="logo-name">PermPoon</span>
                    <span class="logo-tagline">แนะนำบริการเพิ่มพูลรายได้</span>
                </div>
            </RouterLink>

            <!-- Divider -->
            <div class="topbar-divider" />

            <!-- Nav Items -->
            <nav class="topbar-nav">
                <RouterLink
                    to="/"
                    class="nav-tab"
                    :class="{ active: route.path === '/' }"
                    @click="closeSettings"
                >
                    <Lightbulb :size="16" />
                    <span>Smart Advice</span>
                </RouterLink>
                <RouterLink
                    to="/department-search"
                    class="nav-tab"
                    :class="{ active: route.path === '/department-search' }"
                    @click="closeSettings"
                >
                    <Building2 :size="16" />
                    <span>Department Search</span>
                </RouterLink>
            </nav>
        </header>

        <!-- ───────────── MAIN CONTENT ───────────── -->
        <main class="main-content">
            <RouterView />
        </main>

        <!-- ───────────── FOOTER BAR ───────────── -->
        <footer class="footer-bar">
            <span class="footer-copy">&copy; 2026 โรงพยาบาลสระโบสถ์</span>
            <div class="footer-right">
                <!-- Settings Dropdown -->
                <div class="settings-dropdown-wrapper">
                    <button
                        class="footer-btn"
                        :class="{
                            active:
                                route.path.startsWith('/settings') ||
                                settingsOpen,
                        }"
                        @click="toggleSettings"
                    >
                        <Settings :size="13" />
                        <span>การตั้งค่า</span>
                        <ChevronDown
                            :size="11"
                            class="chev"
                            :class="{ open: settingsOpen }"
                        />
                    </button>

                    <Transition name="dropdown">
                        <div
                            v-if="settingsOpen"
                            class="dropdown-menu dropdown-menu-up"
                            @click.stop
                        >
                            <div class="dropdown-label">การตั้งค่าระบบ</div>
                            <RouterLink
                                to="/settings/connection"
                                class="dropdown-item"
                                :class="{
                                    active: route.path.startsWith(
                                        '/settings/connection',
                                    ),
                                }"
                            >
                                <span class="dropdown-icon"
                                    ><Wifi :size="15"
                                /></span>
                                <div>
                                    <div class="dropdown-item-title">
                                        การเชื่อมต่อ
                                    </div>
                                    <div class="dropdown-item-desc">
                                        ตั้งค่าฐานข้อมูลและ API
                                    </div>
                                </div>
                            </RouterLink>
                            <RouterLink
                                to="/settings/pttype"
                                class="dropdown-item"
                                :class="{
                                    active: route.path.startsWith(
                                        '/settings/pttype',
                                    ),
                                }"
                            >
                                <span class="dropdown-icon"
                                    ><Shield :size="15"
                                /></span>
                                <div>
                                    <div class="dropdown-item-title">
                                        สิทธิการรักษา
                                    </div>
                                    <div class="dropdown-item-desc">
                                        จัดการประเภทสิทธิผู้ป่วย
                                    </div>
                                </div>
                            </RouterLink>
                            <RouterLink
                                to="/settings/icode"
                                class="dropdown-item"
                                :class="{
                                    active: route.path.startsWith(
                                        '/settings/icode',
                                    ),
                                }"
                            >
                                <span class="dropdown-icon"
                                    ><Tag :size="15"
                                /></span>
                                <div>
                                    <div class="dropdown-item-title">
                                        ตั้งค่า icode
                                    </div>
                                    <div class="dropdown-item-desc">
                                        รายการรหัสบริการ icode
                                    </div>
                                </div>
                            </RouterLink>
                            <RouterLink
                                to="/settings/export-import"
                                class="dropdown-item"
                                :class="{
                                    active: route.path.startsWith(
                                        '/settings/export-import',
                                    ),
                                }"
                            >
                                <span class="dropdown-icon"
                                    ><ArrowLeftRight :size="15"
                                /></span>
                                <div>
                                    <div class="dropdown-item-title">
                                        ส่งออก / นำเข้า
                                    </div>
                                    <div class="dropdown-item-desc">
                                        สำรองและกู้คืนการตั้งค่า
                                    </div>
                                </div>
                            </RouterLink>
                            <RouterLink
                                to="/settings/department"
                                class="dropdown-item"
                                :class="{
                                    active: route.path.startsWith(
                                        '/settings/department',
                                    ),
                                }"
                            >
                                <span class="dropdown-icon"
                                    ><Building2 :size="15"
                                /></span>
                                <div>
                                    <div class="dropdown-item-title">
                                        แผนกบริการ
                                    </div>
                                    <div class="dropdown-item-desc">
                                        ตั้งค่าแผนกสำหรับค้นหาผู้ป่วย
                                    </div>
                                </div>
                            </RouterLink>
                        </div>
                    </Transition>
                </div>

                <!-- Connection Status -->
                <div
                    class="status-pill"
                    :class="connStore.isConnected ? 'status-ok' : 'status-err'"
                >
                    <span class="status-dot" />
                    <Wifi v-if="connStore.isConnected" :size="12" />
                    <WifiOff v-else :size="12" />
                    <span>{{
                        connStore.isConnected
                            ? "เชื่อมต่อแล้ว"
                            : "ไม่ได้เชื่อมต่อ"
                    }}</span>
                </div>
            </div>
        </footer>
    </div>
</template>

<style scoped>
/* ─── Shell ─────────────────────────────── */
.app-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
    background: var(--surface);
}

/* ─── Topbar ─────────────────────────────── */
.topbar {
    display: flex;
    align-items: center;
    gap: 0;
    height: 58px;
    min-height: 58px;
    background: #ffffff;
    border-bottom: 1px solid var(--hairline);
    padding: 0 20px;
    box-shadow: 0 1px 6px rgba(0, 0, 0, 0.06);
    position: relative;
    z-index: 100;
}

/* ─── Logo ─────────────────────────────── */
.topbar-logo {
    display: flex;
    align-items: center;
    gap: 10px;
    text-decoration: none;
    flex-shrink: 0;
    padding-right: 4px;
}
.logo-icon {
    display: flex;
    align-items: center;
}
.logo-text {
    display: flex;
    flex-direction: column;
    line-height: 1.15;
}
.logo-name {
    font-size: 17px;
    font-weight: 800;
    color: var(--brand-deep);
    letter-spacing: -0.3px;
}
.logo-tagline {
    font-size: 10.5px;
    color: var(--steel);
    font-weight: 400;
}

/* ─── Divider ─────────────────────────── */
.topbar-divider {
    width: 1px;
    height: 28px;
    background: var(--hairline);
    margin: 0 16px;
    flex-shrink: 0;
}

/* ─── Nav ─────────────────────────────── */
.topbar-nav {
    display: flex;
    align-items: center;
    gap: 2px;
    flex: 1;
}

.topbar-right {
    display: flex;
    align-items: center;
    gap: 2px;
    margin-left: auto;
}

.nav-tab {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 7px 14px;
    border-radius: var(--radius-md);
    font-size: 14px;
    font-weight: 500;
    color: var(--slate);
    text-decoration: none;
    cursor: pointer;
    background: transparent;
    border: none;
    transition:
        background 0.15s,
        color 0.15s;
    white-space: nowrap;
    font-family: inherit;
}
.nav-tab:hover {
    background: var(--surface);
    color: var(--charcoal);
}
.nav-tab.active {
    background: #fff7ed;
    color: var(--brand-deep);
}
.nav-tab.active svg {
    color: var(--brand-orange);
}

.chev {
    transition: transform 0.2s;
    color: var(--steel);
    margin-left: 2px;
}
.chev.open {
    transform: rotate(180deg);
}

/* ─── Settings Dropdown ───────────────── */
.settings-dropdown-wrapper {
    position: relative;
}

.dropdown-menu {
    position: absolute;
    bottom: calc(100% + 6px);
    left: auto;
    right: 0;
    min-width: 240px;
    background: #ffffff;
    border: 1px solid var(--hairline);
    border-radius: var(--radius-lg);
    box-shadow:
        0 8px 28px rgba(0, 0, 0, 0.12),
        0 2px 8px rgba(0, 0, 0, 0.06);
    padding: 8px;
    z-index: 1000;
}

.dropdown-label {
    font-size: 10.5px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.6px;
    color: var(--steel);
    padding: 4px 10px 8px;
}

.dropdown-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 9px 10px;
    border-radius: var(--radius-md);
    text-decoration: none;
    color: var(--charcoal);
    transition: background 0.12s;
    cursor: pointer;
}
.dropdown-item:hover {
    background: var(--surface);
}
.dropdown-item.active {
    background: #fff7ed;
    color: var(--brand-deep);
}
.dropdown-item.active .dropdown-icon {
    color: var(--brand-orange);
}

.dropdown-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 30px;
    border-radius: var(--radius-md);
    background: var(--surface);
    color: var(--slate);
    flex-shrink: 0;
}

.dropdown-item-title {
    font-size: 13.5px;
    font-weight: 500;
    line-height: 1.2;
}
.dropdown-item-desc {
    font-size: 11.5px;
    color: var(--steel);
    margin-top: 1px;
}

/* ─── Dropdown Transition ─────────────── */
.dropdown-enter-active,
.dropdown-leave-active {
    transition:
        opacity 0.15s ease,
        transform 0.15s ease;
}
.dropdown-enter-from,
.dropdown-leave-to {
    opacity: 0;
    transform: translateY(-6px);
}

/* ─── Status Bar ─────────────────────── */
.status-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 30px;
    min-height: 30px;
    background: #f8fafc;
    border-bottom: 1px solid var(--hairline);
    padding: 0 20px;
    font-size: 12px;
    color: var(--steel);
    flex-shrink: 0;
    position: relative;
    z-index: 99;
}
.status-bar-copy {
    font-size: 11.5px;
    color: var(--steel);
}
.status-bar-right {
    display: flex;
    align-items: center;
    gap: 8px;
}
.status-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 3px 8px;
    border-radius: var(--radius-md);
    font-size: 12px;
    font-weight: 500;
    color: var(--slate);
    cursor: pointer;
    background: transparent;
    border: none;
    transition:
        background 0.15s,
        color 0.15s;
    white-space: nowrap;
    font-family: inherit;
}
.status-btn:hover {
    background: #eef2f7;
    color: var(--charcoal);
}
.status-btn.active {
    background: #fff7ed;
    color: var(--brand-deep);
}

.status-pill {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11.5px;
    font-weight: 500;
    padding: 3px 8px;
    border-radius: var(--radius-full);
}
.status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
}
.status-ok {
    color: #15803d;
    background: #f0fdf4;
    border: 1px solid #bbf7d0;
}
.status-ok .status-dot {
    background: #22c55e;
    box-shadow: 0 0 0 2px rgba(34, 197, 94, 0.25);
}
.status-err {
    color: #dc2626;
    background: #fef2f2;
    border: 1px solid #fecaca;
}
.status-err .status-dot {
    background: #ef4444;
    box-shadow: 0 0 0 2px rgba(239, 68, 68, 0.25);
}

.hospital-panel {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11.5px;
    font-weight: 600;
    color: var(--slate);
    background: var(--surface);
    border: 1px solid var(--hairline);
    padding: 3px 8px;
    border-radius: var(--radius-full);
}
.hospital-icon {
    color: var(--brand-orange);
}

/* ─── Main Content ────────────────────── */
.main-content {
    flex: 1;
    overflow-y: auto;
    min-width: 0;
}

/* ─── Footer Bar ─────────────────────── */
.footer-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 30px;
    min-height: 30px;
    background: #f8fafc;
    border-top: 1px solid var(--hairline);
    padding: 0 20px;
    font-size: 12px;
    color: var(--steel);
    flex-shrink: 0;
    position: relative;
    z-index: 99;
}
.footer-copy {
    font-size: 11.5px;
    color: var(--steel);
}
.footer-right {
    display: flex;
    align-items: center;
    gap: 8px;
    position: relative;
    z-index: 1000;
}
.footer-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 3px 8px;
    border-radius: var(--radius-md);
    font-size: 12px;
    font-weight: 500;
    color: var(--slate);
    cursor: pointer;
    background: transparent;
    border: none;
    transition:
        background 0.15s,
        color 0.15s;
    white-space: nowrap;
    font-family: inherit;
}
.footer-btn:hover {
    background: #eef2f7;
    color: var(--charcoal);
}
.footer-btn.active {
    background: #fff7ed;
    color: var(--brand-deep);
}
</style>
