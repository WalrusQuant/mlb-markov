import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  DbStatus,
  ImportProgress,
  ImportResult,
  OffenseBundle,
  TeamOption,
} from "./types";

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

export async function getOffenseTransitions(
  season: number,
  teamId?: number | null,
): Promise<OffenseBundle> {
  return await invoke<OffenseBundle>("get_offense_transitions", {
    season,
    teamId: teamId ?? null,
  });
}

export async function getTeams(): Promise<TeamOption[]> {
  return await invoke<TeamOption[]>("get_teams");
}
