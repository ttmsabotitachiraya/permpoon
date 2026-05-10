import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { IcodeConfig } from '../types/icode'

export const useIcodeStore = defineStore('icode', () => {
  const items = ref<IcodeConfig[]>([])

  async function loadAll() {
    try {
      const data = await invoke<IcodeConfig[]>('get_all_icode_configs')
      items.value = data
    } catch (e) {
      console.error(e)
    }
  }

  async function saveIcode(input: Omit<IcodeConfig, 'id'> & { id?: number }) {
    await invoke('save_icode_config', { input })
    await loadAll()
  }

  async function toggleEnabled(id: number, isEnabled: boolean) {
    await invoke('toggle_icode_enabled', { id, isEnabled })
    await loadAll()
  }

  async function removeIcode(id: number) {
    await invoke('remove_icode_config', { id })
    await loadAll()
  }

  return { items, loadAll, saveIcode, toggleEnabled, removeIcode }
})
