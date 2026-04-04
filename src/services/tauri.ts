import { invoke } from "@tauri-apps/api/core";
import type {
  DashboardSummary,
  Release,
  ReleaseInput,
  ReleaseTrackRow,
  Track,
  TrackInput,
  TrackListRow,
} from "../types/models";

export async function initializeApp(): Promise<void> {
  await invoke("initialize_app");
}

export async function getDashboardSummary(): Promise<DashboardSummary> {
  return invoke("get_dashboard_summary");
}

export async function listTracks(): Promise<TrackListRow[]> {
  return invoke("list_tracks");
}

export async function getTrackById(id: number): Promise<Track | null> {
  return invoke("get_track_by_id", { id });
}

export async function createTrack(input: TrackInput): Promise<Track> {
  return invoke("create_track", { input });
}

export async function updateTrack(id: number, input: TrackInput): Promise<Track> {
  return invoke("update_track", { id, input });
}

export async function deleteTrack(id: number): Promise<void> {
  await invoke("delete_track", { id });
}

export async function listAvailableTracks(): Promise<Track[]> {
  return invoke("list_available_tracks");
}

export async function searchTracks(query: string): Promise<TrackListRow[]> {
  return invoke("search_tracks", { query });
}

export async function listReleases(): Promise<Release[]> {
  return invoke("list_releases");
}

export async function getReleaseById(id: number): Promise<Release | null> {
  return invoke("get_release_by_id", { id });
}

export async function createRelease(input: ReleaseInput): Promise<Release> {
  return invoke("create_release", { input });
}

export async function updateRelease(id: number, input: ReleaseInput): Promise<Release> {
  return invoke("update_release", { id, input });
}

export async function deleteRelease(id: number): Promise<void> {
  await invoke("delete_release", { id });
}

export async function listTracksForRelease(releaseId: number): Promise<ReleaseTrackRow[]> {
  return invoke("list_tracks_for_release", { releaseId });
}

export async function assignTrackToRelease(trackId: number, releaseId: number): Promise<void> {
  await invoke("assign_track_to_release", { trackId, releaseId });
}

export async function removeTrackFromRelease(trackId: number, releaseId: number): Promise<void> {
  await invoke("remove_track_from_release", { trackId, releaseId });
}

export async function moveTrackUpInRelease(trackId: number, releaseId: number): Promise<void> {
  await invoke("move_track_up_in_release", { trackId, releaseId });
}

export async function moveTrackDownInRelease(trackId: number, releaseId: number): Promise<void> {
  await invoke("move_track_down_in_release", { trackId, releaseId });
}

export async function setReleaseImage(releaseId: number, sourcePath: string): Promise<Release> {
  return invoke("set_release_image", { releaseId, sourcePath });
}

export async function removeReleaseImage(releaseId: number): Promise<Release> {
  return invoke("remove_release_image", { releaseId });
}

export async function writeTextFile(path: string, contents: string): Promise<void> {
  await invoke("write_text_file", { path, contents });
}


export async function getBackupLocation(): Promise<string | null> {
  return invoke("get_backup_location");
}

export async function setBackupLocation(path: string): Promise<string | null> {
  return invoke("set_backup_location", { path });
}

export async function getBackupOnExit(): Promise<boolean> {
  return invoke("get_backup_on_exit");
}

export async function setBackupOnExit(enabled: boolean): Promise<boolean> {
  return invoke("set_backup_on_exit", { enabled });
}

export async function createBackup(destinationDir: string): Promise<string> {
  return invoke("create_backup", { destinationDir });
}

export async function restoreBackup(backupZipPath: string): Promise<void> {
  await invoke("restore_backup", { backupZipPath });
}
