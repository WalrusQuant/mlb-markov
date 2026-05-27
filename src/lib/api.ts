import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  DbStatus,
  ImportProgress,
  ImportResult,
  OffenseBundle,
  TeamOption,
  PitcherSearchResult,
  PitchSequenceBundle,
} from "./types";

export async function getDbStatus(): Promise<DbStatus> {
  return await invoke<DbStatus>("get_db_status");
}

export async function importSeason(season?: number): Promise<ImportResult> {
  return await invoke<ImportResult>("import_season", {
    season: season ?? null,
  });
}

export async function onImportProgress(
  callback: (progress: ImportProgress) => void,
): Promise<UnlistenFn> {
  return await listen<ImportProgress>("import-progress", (event) => {
    callback(event.payload);
  });
}

export async function getOffenseTransitions(
  season?: number | null,
  teamId?: number | null,
): Promise<OffenseBundle> {
  return await invoke<OffenseBundle>("get_offense_transitions", {
    season: season ?? null,
    teamId: teamId ?? null,
  });
}

export async function getTeams(): Promise<TeamOption[]> {
  return await invoke<TeamOption[]>("get_teams");
}

export async function searchPitchers(
  query: string,
  season?: number,
): Promise<PitcherSearchResult[]> {
  return await invoke<PitcherSearchResult[]>("search_pitchers", {
    query,
    season: season ?? null,
  });
}

export async function getPitchSequences(
  pitcherId: number,
  season?: number,
): Promise<PitchSequenceBundle> {
  return await invoke<PitchSequenceBundle>("get_pitch_sequences", {
    pitcherId,
    season: season ?? null,
  });
}
