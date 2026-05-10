<script setup lang="ts">
import { ref } from 'vue'
import { useConnectionStore } from '../../stores/connectionStore'
import { Wifi, WifiOff, Save, Eye, EyeOff } from 'lucide-vue-next'

const connStore = useConnectionStore()
const showPassword = ref(false)
const saveMsg = ref('')

async function handleTest() {
  await connStore.testConnection()
}

async function handleSave() {
  await connStore.saveConfig()
  saveMsg.value = 'บันทึกการตั้งค่าแล้ว'
  setTimeout(() => saveMsg.value = '', 3000)
}
</script>

<template>
  <div class="conn-settings">
    <div class="card">
      <div class="card-header">
        <Wifi :size="18" style="color:var(--brand-orange)" />
        ตั้งค่าการเชื่อมต่อ MySQL (HOSxP)
      </div>
      <div class="card-body">
        <div class="form-grid">
          <div class="form-group">
            <label class="form-label">MySQL Host</label>
            <input v-model="connStore.config.host" class="form-input" placeholder="127.0.0.1 หรือ IP Server" />
          </div>
          <div class="form-group">
            <label class="form-label">MySQL Port</label>
            <input v-model.number="connStore.config.port" type="number" class="form-input" placeholder="3306" />
          </div>
          <div class="form-group">
            <label class="form-label">Database</label>
            <input v-model="connStore.config.database" class="form-input" placeholder="hos" />
          </div>
          <div class="form-group">
            <label class="form-label">Username</label>
            <input v-model="connStore.config.username" class="form-input" placeholder="username" />
          </div>
          <div class="form-group form-full">
            <label class="form-label">Password</label>
            <div class="password-row">
              <input
                v-model="connStore.config.password"
                :type="showPassword ? 'text' : 'password'"
                class="form-input"
                placeholder="password"
              />
              <button class="btn btn-secondary" @click="showPassword = !showPassword" style="padding:8px 12px">
                <Eye v-if="!showPassword" :size="16" />
                <EyeOff v-else :size="16" />
              </button>
            </div>
          </div>
        </div>

        <div class="divider"></div>

        <div class="action-row">
          <button class="btn btn-secondary" @click="handleTest" :disabled="connStore.isLoading">
            <Wifi :size="16" />
            {{ connStore.isLoading ? 'กำลังทดสอบ...' : 'ทดสอบการเชื่อมต่อ' }}
          </button>
          <button class="btn btn-primary" @click="handleSave">
            <Save :size="16" />
            บันทึก
          </button>
        </div>

        <!-- Status -->
        <div v-if="connStore.isConnected" class="alert alert-success mt-4">
          <Wifi :size="16" />
          เชื่อมต่อสำเร็จ! ฐานข้อมูล HOSxP พร้อมใช้งาน
        </div>
        <div v-else-if="connStore.errorMsg" class="alert alert-error mt-4">
          <WifiOff :size="16" />
          <div>
            <div style="font-weight:500">เชื่อมต่อไม่สำเร็จ</div>
            <div class="text-sm" style="margin-top:4px;font-family:monospace">{{ connStore.errorMsg }}</div>
          </div>
        </div>
        <div v-if="saveMsg" class="alert alert-success mt-4">
          <Save :size="16" />
          {{ saveMsg }}
        </div>
      </div>
    </div>

    <div class="info-card">
      <p class="text-sm text-muted">
        <strong>หมายเหตุ:</strong> การเชื่อมต่อนี้เป็น <strong>อ่านอย่างเดียว (READ-ONLY)</strong>
        โปรแกรมจะไม่เขียนหรือแก้ไขข้อมูลใดๆ ในฐานข้อมูล HOSxP
      </p>
    </div>
  </div>
</template>

<style scoped>
.conn-settings { display: flex; flex-direction: column; gap: 16px; max-width: 600px; }
.form-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; }
.form-full { grid-column: 1 / -1; }
.password-row { display: flex; gap: 8px; }
.password-row .form-input { flex: 1; }
.action-row { display: flex; gap: 8px; }
.mt-4 { margin-top: 16px; }
.info-card {
  padding: 16px; background: var(--surface-soft);
  border: 1px solid #fed7aa; border-radius: var(--radius-md);
}
</style>
