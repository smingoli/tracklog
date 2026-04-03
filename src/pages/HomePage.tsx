import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import {
  backupToGoogleDrive,
  completeGoogleDriveOAuth,
  getDashboardSummary,
  initializeApp,
  restoreLatestFromGoogleDrive,
  startGoogleDriveOAuth
} from "../services/tauri";
import type { DashboardSummary } from "../types/models";

export function HomePage() {
  const [data, setData] = useState<DashboardSummary | null>(null);
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");
  const [folderId, setFolderId] = useState("");
  const [oauthState, setOauthState] = useState("");
  const [connecting, setConnecting] = useState(false);

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

  async function handleConnectGoogleDrive() {
    setMessage("");
    setError("");
    try {
      const state = await startGoogleDriveOAuth();
      setOauthState(state);
      setConnecting(true);
      setMessage("Browser opened for Google sign-in. Waiting for authorization...");
    } catch (e) {
      setError(String(e));
    }
  }

  useEffect(() => {
    if (!connecting || !oauthState) return;
    const timer = window.setInterval(async () => {
      try {
        const done = await completeGoogleDriveOAuth(oauthState);
        if (done) {
          setConnecting(false);
          setOauthState("");
          setMessage("Google Drive connected successfully.");
        }
      } catch (e) {
        setConnecting(false);
        setOauthState("");
        setError(String(e));
      }
    }, 1500);
    return () => window.clearInterval(timer);
  }, [connecting, oauthState]);

  async function handleBackupToGoogleDriveApi() {
    setMessage("");
    setError("");
    try {
      const result = await backupToGoogleDrive(folderId);
      setMessage(result);
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleRestoreFromGoogleDriveApi() {
    setMessage("");
    setError("");
    const confirmed = window.confirm(
      "Restore will replace your current local TrackLog data. Continue?"
    );
    if (!confirmed) return;

    try {
      await restoreLatestFromGoogleDrive(folderId);
      await refreshSummary();
      setMessage("Latest backup restored from Google Drive.");
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
          Uses Google OAuth authorization code flow with PKCE in your system browser.
        </p>
        <div className="form-grid">
          <div className="field full">
            <label>Google Drive Folder ID (Optional)</label>
            <input
              value={folderId}
              onChange={(e) => setFolderId(e.target.value)}
              placeholder="If empty, uses My Drive root"
            />
          </div>
        </div>
        <div className="actions">
          <button className="btn secondary" type="button" onClick={handleConnectGoogleDrive} disabled={connecting}>
            {connecting ? "Connecting..." : "Connect Google Drive"}
          </button>
          <button className="btn" type="button" onClick={handleBackupToGoogleDriveApi}>
            Backup to Google Drive
          </button>
          <button className="btn secondary" type="button" onClick={handleRestoreFromGoogleDriveApi}>
            Restore Latest from Google Drive
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
