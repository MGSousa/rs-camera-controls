// src/composables/StoreApp.ts
import { shallowRef } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Store } from "@tauri-apps/plugin-store";

// A single, shared reactive ref for the store
const store = shallowRef<Store | null>(null);

/**
 * Initializes the store if not already loaded.
 */
export async function initStore() {
  if (store.value) return store.value;

  let storeFile: string = await invoke("get_store");
  store.value = await Store.load(storeFile);
  return store.value;
}

/**
 * Exposes the store and ensures it's initialized before use.
 */
export async function getStore(): Promise<Store> {
  return store.value ?? (await initStore());
}

/**
 * Get content from Store
 */
export async function get(method:string): Promise<any> {
  let res = await getStore();
  return await res.get(method);
}

/**
 * Set content to Store
 */
export async function set(method:string, params:object): Promise<any> {
  let res = await getStore();
  return await res.set(method, params);
}

/**
 * Export shallowRef 
 */
export { store };
