import { useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import { initializeApp, listTracks, searchTracks } from "../services/tauri";
import type { TrackListRow, TrackStatus } from "../types/models";

export function TracksPage() {
  const [tracks, setTracks] = useState<TrackListRow[]>([]);
  const [query, setQuery] = useState("");
  const [statusFilter, setStatusFilter] = useState<"All" | TrackStatus>("All");
  const [availabilityFilter, setAvailabilityFilter] = useState<"All" | "Available" | "Assigned">("All");

  async function reload(q?: string) {
    await initializeApp();
    const rows = (q && q.trim())
      ? await searchTracks(q).catch(() => [])
      : await listTracks().catch(() => []);
    setTracks(rows);
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

  const filtered = useMemo(() => {
    return tracks.filter((track) => {
      const statusOk = statusFilter === "All" || track.status === statusFilter;
      const availabilityOk =
        availabilityFilter === "All" || track.availability === availabilityFilter;
      return statusOk && availabilityOk;
    });
  }, [tracks, statusFilter, availabilityFilter]);

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
              onChange={(e) => setAvailabilityFilter(e.target.value as "All" | "Available" | "Assigned")}
            >
              <option value="All">All</option>
              <option value="Available">Available</option>
              <option value="Assigned">Assigned</option>
            </select>
          </div>
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
