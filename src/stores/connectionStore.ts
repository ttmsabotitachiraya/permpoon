import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface DbConfig {
  host: string;
  port: number;
  database: string;
  username: string;
  password: string;
}

export const useConnectionStore = defineStore("connection", () => {
  const config = ref<DbConfig>({
    host: "127.0.0.1",
    port: 3306,
    database: "hos",
    username: "",
    password: "",
  });
  const isConnected = ref(false);
  const isLoading = ref(false);
  const errorMsg = ref("");

  /** Loads DB config from db.json (credentials are decrypted on the Rust side). */
  async function loadConfig() {
    try {
      const loaded = await invoke<DbConfig>("load_db_config");
      config.value = loaded;
    } catch (e) {
      console.error("loadConfig error:", e);
    }
  }

  /** Saves DB config to db.json (credentials are encrypted on the Rust side). */
  async function saveConfig() {
    await invoke("save_db_config", { config: config.value });
  }

  async function testConnection() {
    isLoading.value = true;
    errorMsg.value = "";
    try {
      await invoke("test_db_connection", { config: config.value });
      isConnected.value = true;
    } catch (e: unknown) {
      isConnected.value = false;
      errorMsg.value = String(e);
    } finally {
      isLoading.value = false;
    }
  }

  return {
    config,
    isConnected,
    isLoading,
    errorMsg,
    loadConfig,
    saveConfig,
    testConnection,
  };
});
