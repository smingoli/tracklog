import { useEffect, useMemo, useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import {
  createTrack,
  deleteTrack,
  getTrackById,
  initializeApp,
  listReleases,
  listTracks,
  removeTrackFromRelease,
  assignTrackToRelease,
  updateTrack,
} from "../services/tauri";
import type { Release, TrackInput, TrackListRow, TrackStatus } from "../types/models";

type Props = {
  mode: "create" | "edit";
};

const emptyForm: TrackInput = {
  internalCode: "",
  title: "",
  status: "Idea",
  description: null,
  lyrics: null,
  notes: null,
  bpm: null,
  key: null,
};

export function TrackDetailPage({ mode }: Props) {
  const { id } = useParams();
  const navigate = useNavigate();
  const [form, setForm] = useState<TrackInput>(emptyForm);
  const [loading, setLoading] = useState(mode === "edit");
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [trackRow, setTrackRow] = useState<TrackListRow | null>(null);
  const [releases, setReleases] = useState<Release[]>([]);
  const [selectedReleaseId, setSelectedReleaseId] = useState<string>("");

  useEffect(() => {
    (async () => {
      await initializeApp();
      const releaseRows = await listReleases().catch(() => []);
      setReleases(releaseRows);
      if (mode === "edit" && id) {
        try {
          const track = await getTrackById(Number(id));
          const listRows = await listTracks().catch(() => []);
          const row = listRows.find((x) => x.id === Number(id)) ?? null;
          setTrackRow(row);
          if (!track) {
            setError("Track not found");
            return;
          }
          setForm({
            internalCode: track.internalCode,
            title: track.title,
            status: track.status,
            description: track.description,
            lyrics: track.lyrics,
            notes: track.notes,
            bpm: track.bpm,
            key: track.key,
          });
        } catch (e) {
          setError(String(e));
        } finally {
          setLoading(false);
        }
      } else {
        setLoading(false);
      }
    })();
  }, [mode, id]);

  const canSave = useMemo(() => {
    return form.internalCode.trim() && form.title.trim();
  }, [form]);

  function patch<K extends keyof TrackInput>(key: K, value: TrackInput[K]) {
    setForm((prev) => ({ ...prev, [key]: value }));
  }

  async function refreshAssignmentState(trackId: number) {
    const rows = await listTracks().catch(() => []);
    setTrackRow(rows.find((x) => x.id === trackId) ?? null);
  }

  async function handleSave() {
    setError(null);
    setMessage(null);
    try {
      await initializeApp();
      if (mode === "create") {
        const created = await createTrack(form);
        navigate(`/tracks/${created.id}`);
        return;
      }
      if (!id) return;
      await updateTrack(Number(id), form);
      await refreshAssignmentState(Number(id));
      setMessage("Track saved.");
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleDelete() {
    if (!id) return;
    const confirmed = window.confirm("Delete this track?");
    if (!confirmed) return;
    try {
      await deleteTrack(Number(id));
      navigate("/tracks");
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleAssign() {
    if (!id || !selectedReleaseId) return;
    try {
      await assignTrackToRelease(Number(id), Number(selectedReleaseId));
      await refreshAssignmentState(Number(id));
      setMessage("Track assigned.");
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleRemoveFromRelease() {
    if (!id || !trackRow?.assignedReleaseId) return;
    const confirmed = window.confirm("Remove this track from its release?");
    if (!confirmed) return;
    try {
      await removeTrackFromRelease(Number(id), trackRow.assignedReleaseId);
      await refreshAssignmentState(Number(id));
      setMessage("Track removed from release.");
    } catch (e) {
      setError(String(e));
    }
  }

  if (loading) return <p>Loading...</p>;

  return (
    <div>
      <div className="page-header">
        <h2>{mode === "create" ? "New Track" : "Track Detail"}</h2>
      </div>

      <div className="panel">
        <div className="form-grid">
          <div className="field">
            <label>Internal Code</label>
            <input value={form.internalCode} onChange={(e) => patch("internalCode", e.target.value)} />
          </div>
          <div className="field">
            <label>Title</label>
            <input value={form.title} onChange={(e) => patch("title", e.target.value)} />
          </div>
          <div className="field">
            <label>Status</label>
            <select value={form.status} onChange={(e) => patch("status", e.target.value as TrackStatus)}>
              <option>Idea</option>
              <option>Draft</option>
              <option>In Progress</option>
              <option>Final</option>
            </select>
          </div>
          <div className="field">
            <label>BPM</label>
            <input
              type="number"
              min="1"
              value={form.bpm ?? ""}
              onChange={(e) => patch("bpm", e.target.value ? Number(e.target.value) : null)}
            />
          </div>
          <div className="field">
            <label>Key</label>
            <input value={form.key ?? ""} onChange={(e) => patch("key", e.target.value || null)} />
          </div>
          <div className="field full">
            <label>Description</label>
            <textarea value={form.description ?? ""} onChange={(e) => patch("description", e.target.value || null)} />
          </div>
          <div className="field full">
            <label>Lyrics</label>
            <textarea value={form.lyrics ?? ""} onChange={(e) => patch("lyrics", e.target.value || null)} />
          </div>
          <div className="field full">
            <label>Notes</label>
            <textarea value={form.notes ?? ""} onChange={(e) => patch("notes", e.target.value || null)} />
          </div>
        </div>

        <div className="actions" style={{ marginTop: 16 }}>
          <button disabled={!canSave} onClick={handleSave}>Save</button>
          <button className="secondary" onClick={() => navigate("/tracks")}>Back</button>
          {mode === "edit" && <button className="danger" onClick={handleDelete}>Delete</button>}
        </div>

        {error && <p className="muted">{error}</p>}
        {message && <p className="muted">{message}</p>}
      </div>

      {mode === "edit" && id && (
        <div className="panel">
          <h3>Release Assignment</h3>
          {!trackRow?.assignedReleaseId ? (
            <>
              <p className="muted">Available</p>
              <div className="actions">
                <select
                  className="small-input"
                  value={selectedReleaseId}
                  onChange={(e) => setSelectedReleaseId(e.target.value)}
                >
                  <option value="">Select release</option>
                  {releases.map((release) => (
                    <option key={release.id} value={release.id}>
                      {release.internalCode} - {release.title}
                    </option>
                  ))}
                </select>
                <button disabled={!selectedReleaseId} onClick={handleAssign}>Assign to Release</button>
              </div>
            </>
          ) : (
            <>
              <p>
                Assigned to{" "}
                <Link to={`/releases/${trackRow.assignedReleaseId}`}>
                  {trackRow.assignedReleaseTitle}
                </Link>
              </p>
              <div className="actions">
                <Link className="btn secondary" to={`/releases/${trackRow.assignedReleaseId}`}>Open Release</Link>
                <button className="danger" onClick={handleRemoveFromRelease}>Remove from Release</button>
              </div>
            </>
          )}
        </div>
      )}
    </div>
  );
}
