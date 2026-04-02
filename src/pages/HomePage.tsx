import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { getDashboardSummary, initializeApp } from "../services/tauri";
import type { DashboardSummary } from "../types/models";

export function HomePage() {
  const [data, setData] = useState<DashboardSummary | null>(null);

  useEffect(() => {
    (async () => {
      await initializeApp();
      const summary = await getDashboardSummary().catch(() => null);
      setData(summary);
    })();
  }, []);

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
