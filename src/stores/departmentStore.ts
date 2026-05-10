import { defineStore } from "pinia";
import { invoke } from "@tauri-apps/api/core";

export interface DepartmentConfig {
    id: number;
    depcode: string;
    department: string;
    is_enabled: boolean;
}

export const useDepartmentStore = defineStore("department", {
    state: () => ({
        items: [] as DepartmentConfig[],
        isLoading: false,
    }),
    getters: {
        enabledItems: (state) => state.items.filter((i) => i.is_enabled),
    },
    actions: {
        async loadAll() {
            this.isLoading = true;
            try {
                const data = await invoke<DepartmentConfig[]>(
                    "get_all_department_configs",
                );
                this.items = data;
            } finally {
                this.isLoading = false;
            }
        },
        async saveDept(input: { depcode: string; department: string; is_enabled: boolean }) {
            await invoke("save_department_config", { input });
            await this.loadAll();
        },
        async toggleEnabled(id: number, isEnabled: boolean) {
            await invoke("toggle_department_enabled", { id, is_enabled: isEnabled });
            await this.loadAll();
        },
        async removeDept(id: number) {
            await invoke("remove_department_config", { id });
            await this.loadAll();
        },
    },
});