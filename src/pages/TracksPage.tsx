import { useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import { initializeApp, listReleases, listTracks, searchTracks } from "../services/tauri";
import type { Release, TrackListRow, TrackStatus } from "../types/models";

type AvailabilityFilter = "All" | "Available" | "Assigned";
type TrackFilters = {
  status: "All" | TrackStatus;
  availability: AvailabilityFilter;
  releaseId: "All" | string;
};

const STORAGE_KEY = "tracklog:tracks:filters";

function loadPersistedFilters(): TrackFilters {
  if (typeof window === "undefined") {
    return { status: "All", availability: "All", releaseId: "All" };
  }

  const raw = window.localStorage.getItem(STORAGE_KEY);
  if (!raw) {
    return { status: "All", availability: "All", releaseId: "All" };
  }

  try {
    const parsed = JSON.parse(raw) as Partial<TrackFilters>;
    const status =
      parsed.status === "Idea" ||
      parsed.status === "Draft" ||
      parsed.status === "In Progress" ||
      parsed.status === "Final" ||
      parsed.status === "All"
        ? parsed.status
        : "All";

    const availability =
      parsed.availability === "Available" ||
      parsed.availability === "Assigned" ||
      parsed.availability === "All"
        ? parsed.availability
        : "All";

    const releaseId =
      parsed.releaseId === "All" || typeof parsed.releaseId === "string"
        ? parsed.releaseId
        : "All";

    return {
      status,
      availability,
      releaseId,
    };
  } catch {
    return { status: "All", availability: "All", releaseId: "All" };
  }
}

export function TracksPage() {
  const persistedFilters = loadPersistedFilters();

  const [tracks, setTracks] = useState<TrackListRow[]>([]);
  const [assignedReleases, setAssignedReleases] = useState<Release[]>([]);
  const [query, setQuery] = useState("");
  const [statusFilter, setStatusFilter] = useState<"All" | TrackStatus>(persistedFilters.status);
  const [availabilityFilter, setAvailabilityFilter] = useState<AvailabilityFilter>(persistedFilters.availability);
  const [releaseFilter, setReleaseFilter] = useState<"All" | string>(persistedFilters.releaseId);

  async function reload(q?: string) {
    await initializeApp();

    const [rows, releases] = await Promise.all([
      (q && q.trim()) ? searchTracks(q).catch(() => []) : listTracks().catch(() => []),
      listReleases().catch(() => []),
    ]);

    setTracks(rows);
    setAssignedReleases(releases.filter((release) => (release.trackCount ?? 0) > 0));
  }

  useEffect(() => {
    reload();
  }, []);

  useEffect(() => {
    const handle = setTimeout(() => {
      reload(query);
    }, 250);
    return () => clearTimeout(handle);
  }, [query]);

  useEffect(() => {
    if (availabilityFilter !== "Assigned") {
      setReleaseFilter("All");
    }
  }, [availabilityFilter]);

  useEffect(() => {
    if (releaseFilter === "All") {
      return;
    }

    const releaseStillExists = assignedReleases.some((release) => String(release.id) === releaseFilter);
    if (!releaseStillExists) {
      setReleaseFilter("All");
    }
  }, [assignedReleases, releaseFilter]);

  useEffect(() => {
    if (typeof window === "undefined") {
      return;
    }

    const stateToPersist: TrackFilters = {
      status: statusFilter,
      availability: availabilityFilter,
      releaseId: availabilityFilter === "Assigned" ? releaseFilter : "All",
    };

    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(stateToPersist));
  }, [availabilityFilter, releaseFilter, statusFilter]);

  const filtered = useMemo(() => {
    return tracks.filter((track) => {
      const statusOk = statusFilter === "All" || track.status === statusFilter;
      const availabilityOk =
        availabilityFilter === "All" || track.availability === availabilityFilter;
      const releaseOk =
        availabilityFilter !== "Assigned" ||
        releaseFilter === "All" ||
        String(track.assignedReleaseId) === releaseFilter;

      return statusOk && availabilityOk && releaseOk;
    });
  }, [tracks, statusFilter, availabilityFilter, releaseFilter]);

  return (
    <div>
      <div className="page-header">
        <h2>Tracks</h2>
        <Link className="btn" to="/tracks/new">New Track</Link>
      </div>

      <div className="panel">
        <div className="form-grid">
          <div className="field">
            <label>Search</label>
            <input
              placeholder="Search by code or title"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
            />
          </div>
          <div className="field">
            <label>Status</label>
            <select value={statusFilter} onChange={(e) => setStatusFilter(e.target.value as "All" | TrackStatus)}>
              <option value="All">All</option>
              <option>Idea</option>
              <option>Draft</option>
              <option>In Progress</option>
              <option>Final</option>
            </select>
          </div>
          <div className="field">
            <label>Availability</label>
            <select
              value={availabilityFilter}
              onChange={(e) => setAvailabilityFilter(e.target.value as AvailabilityFilter)}
            >
              <option value="All">All</option>
              <option value="Available">Available</option>
              <option value="Assigned">Assigned</option>
            </select>
          </div>
          {availabilityFilter === "Assigned" && (
            <div className="field">
              <label>Release</label>
              <select
                value={releaseFilter}
                onChange={(e) => setReleaseFilter(e.target.value)}
              >
                <option value="All">All assigned releases</option>
                {assignedReleases.map((release) => (
                  <option key={release.id} value={String(release.id)}>
                    {release.title} ({release.internalCode})
                  </option>
                ))}
              </select>
            </div>
          )}
        </div>
      </div>

      <table className="table">
        <thead>
          <tr>
            <th>Internal Code</th>
            <th>Title</th>
            <th>Status</th>
            <th>BPM</th>
            <th>Key</th>
            <th>Assigned Release</th>
            <th>Last Updated</th>
          </tr>
        </thead>
        <tbody>
          {filtered.map((track) => (
            <tr key={track.id}>
              <td><Link to={`/tracks/${track.id}`}>{track.internalCode}</Link></td>
              <td>{track.title}</td>
              <td>{track.status}</td>
              <td>{track.bpm ?? ""}</td>
              <td>{track.key ?? ""}</td>
              <td>
                {track.assignedReleaseId ? (
                  <Link to={`/releases/${track.assignedReleaseId}`}>{track.assignedReleaseTitle}</Link>
                ) : (
                  <span className="badge">Available</span>
                )}
              </td>
              <td>{track.updatedAt}</td>
            </tr>
          ))}
          {filtered.length === 0 && (
            <tr>
              <td colSpan={7} className="muted">No tracks found.</td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}
