import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { open } from "@tauri-apps/plugin-dialog";
import {
  backupToGoogleDriveFolder,
  getDashboardSummary,
  initializeApp,
  restoreFromGoogleDriveBackup
} from "../services/tauri";
import type { DashboardSummary } from "../types/models";

export function HomePage() {
  const [data, setData] = useState<DashboardSummary | null>(null);
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");

  useEffect(() => {
    (async () => {
      await initializeApp();
      const summary = await getDashboardSummary().catch(() => null);
      setData(summary);
    })();
  }, []);

  async function refreshSummary() {
    const summary = await getDashboardSummary().catch(() => null);
    setData(summary);
  }

  async function handleBackupToGoogleDrive() {
    setMessage("");
    setError("");
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select your Google Drive folder"
      });
      if (!selected || Array.isArray(selected)) return;
      const backupPath = await backupToGoogleDriveFolder(selected);
      setMessage(`Backup created: ${backupPath}`);
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleRestoreFromGoogleDrive() {
    setMessage("");
    setError("");
    const confirmed = window.confirm(
      "Restore will replace your current local TrackLog data. Continue?"
    );
    if (!confirmed) return;

    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select a TrackLog backup folder from Google Drive"
      });
      if (!selected || Array.isArray(selected)) return;
      await restoreFromGoogleDriveBackup(selected);
      await refreshSummary();
      setMessage("Backup restored successfully.");
    } catch (e) {
      setError(String(e));
    }
  }

  return (
    <div>
      <div className="page-header">
        <h2>Home</h2>
        <div className="actions">
          <Link className="btn" to="/tracks/new">New Track</Link>
          <Link className="btn secondary" to="/releases/new">New Release</Link>
        </div>
      </div>

      <div className="card-grid">
        <div className="card">
          <h3>Total Tracks</h3>
          <div>{data?.totalTracks ?? "-"}</div>
        </div>
        <div className="card">
          <h3>Available Tracks</h3>
          <div>{data?.availableTracks ?? "-"}</div>
        </div>
        <div className="card">
          <h3>Total Releases</h3>
          <div>{data?.totalReleases ?? "-"}</div>
        </div>
      </div>

      <div className="panel">
        <h3>Google Drive Backup</h3>
        <p className="muted">
          Choose your Google Drive desktop-sync folder to create a backup, or restore from an existing backup folder.
        </p>
        <div className="actions">
          <button className="btn" type="button" onClick={handleBackupToGoogleDrive}>
            Backup to Google Drive
          </button>
          <button className="btn secondary" type="button" onClick={handleRestoreFromGoogleDrive}>
            Restore from Google Drive
          </button>
        </div>
        {message && <div className="success">{message}</div>}
        {error && <div className="error">{error}</div>}
      </div>

      <div className="panel">
        <h3>Recently Updated Tracks</h3>
        <div className="inline-list">
          {(data?.recentTracks ?? []).map((track) => (
            <div key={track.id}>
              <Link to={`/tracks/${track.id}`}>
                <strong>{track.internalCode}</strong> - {track.title}
              </Link>
            </div>
          ))}
          {(data?.recentTracks?.length ?? 0) === 0 && <div className="muted">No tracks yet.</div>}
        </div>
      </div>

      <div className="panel">
        <h3>Recently Updated Releases</h3>
        <div className="inline-list">
          {(data?.recentReleases ?? []).map((release) => (
            <div key={release.id}>
              <Link to={`/releases/${release.id}`}>
                <strong>{release.internalCode}</strong> - {release.title}
              </Link>
            </div>
          ))}
          {(data?.recentReleases?.length ?? 0) === 0 && <div className="muted">No releases yet.</div>}
        </div>
      </div>
    </div>
  );
}
