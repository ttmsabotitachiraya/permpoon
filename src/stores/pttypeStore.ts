import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { PttypeGroup } from '../types/pttype'

export const usePttypeStore = defineStore('pttype', () => {
  const groups = ref<PttypeGroup[]>([])

  async function loadAll() {
    try {
      const data = await invoke<PttypeGroup[]>('get_all_pttype_groups')
      groups.value = data
    } catch (e) {
      console.error(e)
    }
  }

  async function addGroup(alias: string, hipdata_code: string) {
    await invoke('add_pttype_group', { alias, hipdataCode: hipdata_code })
    await loadAll()
  }

  async function removeGroup(id: number) {
    await invoke('remove_pttype_group', { id })
    await loadAll()
  }

  return { groups, loadAll, addGroup, removeGroup }
})
