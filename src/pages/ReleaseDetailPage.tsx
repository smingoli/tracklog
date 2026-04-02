import { useEffect, useMemo, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { useNavigate, useParams } from "react-router-dom";
import { convertFileSrc } from "@tauri-apps/api/core";
import {
  assignTrackToRelease,
  createRelease,
  deleteRelease,
  getReleaseById,
  initializeApp,
  listAvailableTracks,
  listTracksForRelease,
  moveTrackDownInRelease,
  moveTrackUpInRelease,
  removeReleaseImage,
  removeTrackFromRelease,
  setReleaseImage,
  updateRelease,
} from "../services/tauri";
import type {
  ReleaseInput,
  ReleaseStatus,
  ReleaseTrackRow,
  ReleaseType,
  Track,
} from "../types/models";

type Props = {
  mode: "create" | "edit";
};

const emptyForm: ReleaseInput = {
  internalCode: "",
  title: "",
  type: "Album",
  status: "Planned",
  description: null,
  imagePath: null,
};

export function ReleaseDetailPage({ mode }: Props) {
  const { id } = useParams();
  const navigate = useNavigate();
  const [form, setForm] = useState<ReleaseInput>(emptyForm);
  const [tracks, setTracks] = useState<ReleaseTrackRow[]>([]);
  const [availableTracks, setAvailableTracks] = useState<Track[]>([]);
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [addQuery, setAddQuery] = useState("");
  const [loading, setLoading] = useState(mode === "edit");
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);

  async function reloadTracks(releaseId: number) {
    const releaseTracks = await listTracksForRelease(releaseId).catch(() => []);
    setTracks(releaseTracks);
  }

  async function reloadAvailableTracks() {
    const rows = await listAvailableTracks().catch(() => []);
    setAvailableTracks(rows);
  }

  useEffect(() => {
    if (mode !== "edit" || !id) {
      setLoading(false);
      return;
    }
    (async () => {
      try {
        await initializeApp();
        const release = await getReleaseById(Number(id));
        if (!release) {
          setError("Release not found");
          return;
        }
        setForm({
          internalCode: release.internalCode,
          title: release.title,
          type: release.type,
          status: release.status,
          description: release.description,
          imagePath: release.imagePath,
        });
        await reloadTracks(Number(id));
        await reloadAvailableTracks();
      } catch (e) {
        setError(String(e));
      } finally {
        setLoading(false);
      }
    })();
  }, [mode, id]);

  const canSave = useMemo(() => form.internalCode.trim() && form.title.trim(), [form]);

  const filteredAvailableTracks = useMemo(() => {
    const q = addQuery.trim().toLowerCase();
    if (!q) return availableTracks;
    return availableTracks.filter((track) =>
      track.internalCode.toLowerCase().includes(q) ||
      track.title.toLowerCase().includes(q)
    );
  }, [availableTracks, addQuery]);

  function patch<K extends keyof ReleaseInput>(key: K, value: ReleaseInput[K]) {
    setForm((prev) => ({ ...prev, [key]: value }));
  }

  async function handleSave() {
    try {
      setError(null);
      setMessage(null);
      await initializeApp();
      if (mode === "create") {
        const created = await createRelease(form);
        navigate(`/releases/${created.id}`);
        return;
      }
      if (!id) return;
      const updated = await updateRelease(Number(id), form);
      setForm({
        internalCode: updated.internalCode,
        title: updated.title,
        type: updated.type,
        status: updated.status,
        description: updated.description,
        imagePath: updated.imagePath,
      });
      setMessage("Release saved.");
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleDelete() {
    if (!id) return;
    const confirmed = window.confirm("Delete this release?");
    if (!confirmed) return;
    try {
      await deleteRelease(Number(id));
      navigate("/releases");
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleAddTrack(trackId: number) {
    if (!id) return;
    try {
      await assignTrackToRelease(trackId, Number(id));
      await reloadTracks(Number(id));
      await reloadAvailableTracks();
      setMessage("Track added.");
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleRemoveTrack(trackId: number) {
    if (!id) return;
    const confirmed = window.confirm("Remove this track from the release?");
    if (!confirmed) return;
    try {
      await removeTrackFromRelease(trackId, Number(id));
      await reloadTracks(Number(id));
      await reloadAvailableTracks();
      setMessage("Track removed.");
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleMoveUp(trackId: number) {
    if (!id) return;
    try {
      await moveTrackUpInRelease(trackId, Number(id));
      await reloadTracks(Number(id));
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleMoveDown(trackId: number) {
    if (!id) return;
    try {
      await moveTrackDownInRelease(trackId, Number(id));
      await reloadTracks(Number(id));
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleChooseImage() {
    if (!id) return;
    try {
      const selected = await open({
        multiple: false,
        filters: [
          { name: "Images", extensions: ["png", "jpg", "jpeg", "webp"] }
        ]
      });
      if (!selected || Array.isArray(selected)) return;
      const updated = await setReleaseImage(Number(id), selected);
      setForm((prev) => ({ ...prev, imagePath: updated.imagePath }));
      setMessage("Release image updated.");
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleRemoveImage() {
    if (!id) return;
    const confirmed = window.confirm("Remove the release image?");
    if (!confirmed) return;
    try {
      const updated = await removeReleaseImage(Number(id));
      setForm((prev) => ({ ...prev, imagePath: updated.imagePath }));
      setMessage("Release image removed.");
    } catch (e) {
      setError(String(e));
    }
  }

  if (loading) {
    return <p>Loading...</p>;
  }

  return (
    <div>
      <div className="page-header">
        <h2>{mode === "create" ? "New Release" : "Release Detail"}</h2>
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
            <label>Type</label>
            <select value={form.type} onChange={(e) => patch("type", e.target.value as ReleaseType)}>
              <option>Album</option>
              <option>EP</option>
              <option>Single</option>
            </select>
          </div>
          <div className="field">
            <label>Status</label>
            <select value={form.status} onChange={(e) => patch("status", e.target.value as ReleaseStatus)}>
              <option>Planned</option>
              <option>In Progress</option>
              <option>Released</option>
            </select>
          </div>
          <div className="field full">
            <label>Description</label>
            <textarea value={form.description ?? ""} onChange={(e) => patch("description", e.target.value || null)} />
          </div>
          <div className="field full">
            <label>Image</label>
            {form.imagePath ? (
              <>
                <img
                  className="image-preview"
                  src={convertFileSrc(form.imagePath)}
                  alt="Release art"
                />
                <p className="muted">{form.imagePath}</p>
              </>
            ) : (
              <p className="muted">No image assigned.</p>
            )}
          </div>
        </div>

        <div className="actions" style={{ marginTop: 16 }}>
          <button disabled={!canSave} onClick={handleSave}>Save</button>
          <button className="secondary" onClick={() => navigate("/releases")}>Back</button>
          {mode === "edit" && <button className="secondary" onClick={handleChooseImage}>{form.imagePath ? "Replace Image" : "Upload Image"}</button>}
          {mode === "edit" && form.imagePath && <button className="danger" onClick={handleRemoveImage}>Remove Image</button>}
          {mode === "edit" && <button className="danger" onClick={handleDelete}>Delete</button>}
        </div>

        {error && <p className="muted">{error}</p>}
        {message && <p className="muted">{message}</p>}
      </div>

      {mode === "edit" && id && (
        <div className="panel">
          <div className="page-header">
            <h3>Tracks</h3>
            <button onClick={() => { setShowAddDialog(true); reloadAvailableTracks(); }}>Add Track</button>
          </div>

          <table className="table">
            <thead>
              <tr>
                <th>#</th>
                <th>Internal Code</th>
                <th>Title</th>
                <th>Status</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {tracks.map((track, idx) => (
                <tr key={track.trackId}>
                  <td>{track.trackOrder}</td>
                  <td>{track.internalCode}</td>
                  <td>{track.title}</td>
                  <td>{track.status}</td>
                  <td>
                    <div className="row-actions">
                      <button className="secondary" disabled={idx === 0} onClick={() => handleMoveUp(track.trackId)}>Move Up</button>
                      <button className="secondary" disabled={idx === tracks.length - 1} onClick={() => handleMoveDown(track.trackId)}>Move Down</button>
                      <button className="danger" onClick={() => handleRemoveTrack(track.trackId)}>Remove</button>
                    </div>
                  </td>
                </tr>
              ))}
              {tracks.length === 0 && (
                <tr>
                  <td colSpan={5} className="muted">No tracks assigned yet.</td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      )}

      {showAddDialog && (
        <div className="dialog-backdrop" onClick={() => setShowAddDialog(false)}>
          <div className="dialog" onClick={(e) => e.stopPropagation()}>
            <div className="page-header">
              <h3>Add Track</h3>
              <button className="secondary" onClick={() => setShowAddDialog(false)}>Close</button>
            </div>
            <div className="field">
              <label>Search available tracks</label>
              <input
                placeholder="Search by code or title"
                value={addQuery}
                onChange={(e) => setAddQuery(e.target.value)}
              />
            </div>
            <table className="table" style={{ marginTop: 16 }}>
              <thead>
                <tr>
                  <th>Internal Code</th>
                  <th>Title</th>
                  <th>Status</th>
                  <th>BPM</th>
                  <th>Key</th>
                  <th>Action</th>
                </tr>
              </thead>
              <tbody>
                {filteredAvailableTracks.map((track) => (
                  <tr key={track.id}>
                    <td>{track.internalCode}</td>
                    <td>{track.title}</td>
                    <td>{track.status}</td>
                    <td>{track.bpm ?? ""}</td>
                    <td>{track.key ?? ""}</td>
                    <td>
                      <button onClick={() => handleAddTrack(track.id)}>Add</button>
                    </td>
                  </tr>
                ))}
                {filteredAvailableTracks.length === 0 && (
                  <tr>
                    <td colSpan={6} className="muted">No available tracks found.</td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  );
}
