import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { DbStatus, ImportProgress, ImportResult } from "./types";

export async function getDbStatus(): Promise<DbStatus> {
  return await invoke<DbStatus>("get_db_status");
}

export async function importSeason(season: number): Promise<ImportResult> {
  return await invoke<ImportResult>("import_season", { season });
}

export async function onImportProgress(
  callback: (progress: ImportProgress) => void,
): Promise<UnlistenFn> {
  return await listen<ImportProgress>("import-progress", (event) => {
    callback(event.payload);
  });
}
