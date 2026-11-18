import { useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import { initializeApp, listReleases } from "../services/tauri";
import type { Release, ReleaseStatus, ReleaseType } from "../types/models";

type ReleaseFilters = {
  type: "All" | ReleaseType;
  status: "All" | ReleaseStatus;
};

const STORAGE_KEY = "tracklog:releases:filters";

function loadPersistedFilters(): ReleaseFilters {
  if (typeof window === "undefined") {
    return { type: "All", status: "All" };
  }

  const raw = window.localStorage.getItem(STORAGE_KEY);
  if (!raw) {
    return { type: "All", status: "All" };
  }

  try {
    const parsed = JSON.parse(raw) as Partial<ReleaseFilters>;
    const type =
      parsed.type === "Album" ||
      parsed.type === "EP" ||
      parsed.type === "Single" ||
      parsed.type === "All"
        ? parsed.type
        : "All";

    const status =
      parsed.status === "Planned" ||
      parsed.status === "In Progress" ||
      parsed.status === "Released" ||
      parsed.status === "All"
        ? parsed.status
        : "All";

    return { type, status };
  } catch {
    return { type: "All", status: "All" };
  }
}

export function ReleasesPage() {
  const persistedFilters = loadPersistedFilters();

  const [releases, setReleases] = useState<Release[]>([]);
  const [typeFilter, setTypeFilter] = useState<"All" | ReleaseType>(persistedFilters.type);
  const [statusFilter, setStatusFilter] = useState<"All" | ReleaseStatus>(persistedFilters.status);

  useEffect(() => {
    (async () => {
      await initializeApp();
      const rows = await listReleases().catch(() => []);
      setReleases(rows);
    })();
  }, []);

  useEffect(() => {
    if (typeof window === "undefined") {
      return;
    }

    const stateToPersist: ReleaseFilters = {
      type: typeFilter,
      status: statusFilter,
    };

    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(stateToPersist));
  }, [typeFilter, statusFilter]);

  const filtered = useMemo(() => {
    return releases.filter((release) => {
      const typeOk = typeFilter === "All" || release.type === typeFilter;
      const statusOk = statusFilter === "All" || release.status === statusFilter;
      return typeOk && statusOk;
    });
  }, [releases, typeFilter, statusFilter]);

  return (
    <div>
      <div className="page-header">
        <h2>Releases</h2>
        <Link className="btn" to="/releases/new">New Release</Link>
      </div>

      <div className="panel">
        <div className="form-grid">
          <div className="field">
            <label>Type</label>
            <select value={typeFilter} onChange={(e) => setTypeFilter(e.target.value as "All" | ReleaseType)}>
              <option value="All">All</option>
              <option value="Album">Album</option>
              <option value="EP">EP</option>
              <option value="Single">Single</option>
            </select>
          </div>
          <div className="field">
            <label>Status</label>
            <select value={statusFilter} onChange={(e) => setStatusFilter(e.target.value as "All" | ReleaseStatus)}>
              <option value="All">All</option>
              <option value="Planned">Planned</option>
              <option value="In Progress">In Progress</option>
              <option value="Released">Released</option>
            </select>
          </div>
        </div>
      </div>

      <table className="table">
        <thead>
          <tr>
            <th>Internal Code</th>
            <th>Title</th>
            <th>Type</th>
            <th>Status</th>
            <th>Number of Tracks</th>
            <th>Last Updated</th>
          </tr>
        </thead>
        <tbody>
          {filtered.map((release) => (
            <tr key={release.id}>
              <td><Link to={`/releases/${release.id}`}>{release.internalCode}</Link></td>
              <td>{release.title}</td>
              <td>{release.type}</td>
              <td>{release.status}</td>
              <td>{release.trackCount ?? 0}</td>
              <td>{release.updatedAt}</td>
            </tr>
          ))}
          {filtered.length === 0 && (
            <tr>
              <td colSpan={6} className="muted">No releases found.</td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}
