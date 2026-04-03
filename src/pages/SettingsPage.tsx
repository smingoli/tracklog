import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import {
  createBackup,
  getBackupLocation,
  initializeApp,
  restoreBackup,
  setBackupLocation,
} from "../services/tauri";

export function SettingsPage() {
  const [backupLocation, setBackupLocationState] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  useEffect(() => {
    (async () => {
      await initializeApp();
      const location = await getBackupLocation().catch(() => null);
      setBackupLocationState(location);
    })();
  }, []);

  async function chooseBackupLocation() {
    setErrorMessage(null);
    setStatusMessage(null);

    const selected = await open({
      directory: true,
      multiple: false,
      title: "Choose backup location",
    });

    if (!selected || Array.isArray(selected)) {
      return;
    }

    setBusy(true);
    try {
      const saved = await setBackupLocation(selected);
      setBackupLocationState(saved);
      setStatusMessage("Backup location saved.");
    } catch (err) {
      setErrorMessage(err instanceof Error ? err.message : "Could not save backup location.");
    } finally {
      setBusy(false);
    }
  }

  async function runBackup() {
    if (!backupLocation) {
      setErrorMessage("Please choose a backup location first.");
      return;
    }

    setBusy(true);
    setErrorMessage(null);
    setStatusMessage(null);
    try {
      const zipPath = await createBackup(backupLocation);
      setStatusMessage(`Backup created: ${zipPath}`);
    } catch (err) {
      setErrorMessage(err instanceof Error ? err.message : "Backup failed.");
    } finally {
      setBusy(false);
    }
  }

  async function runRestore() {
    setErrorMessage(null);
    setStatusMessage(null);

    const selected = await open({
      directory: false,
      multiple: false,
      title: "Select backup ZIP",
      filters: [{ name: "ZIP", extensions: ["zip"] }],
    });

    if (!selected || Array.isArray(selected)) {
      return;
    }

    setBusy(true);
    try {
      await restoreBackup(selected);
      setStatusMessage("Restore complete. Restart the app to ensure all data is reloaded.");
    } catch (err) {
      setErrorMessage(err instanceof Error ? err.message : "Restore failed.");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div>
      <div className="page-header">
        <h2>Settings</h2>
      </div>

      <div className="panel">
        <h3>Backup</h3>
        <p className="muted">Choose where backups are stored, create a zipped backup, or restore from a backup ZIP.</p>

        <div className="backup-row">
          <div className="field full">
            <label>Backup location</label>
            <input value={backupLocation ?? ""} readOnly placeholder="No backup location selected" />
          </div>

          <div className="actions">
            <button type="button" className="secondary" onClick={chooseBackupLocation} disabled={busy}>
              Choose Location
            </button>
            <button type="button" onClick={runBackup} disabled={busy || !backupLocation}>
              Backup Now
            </button>
            <button type="button" className="danger" onClick={runRestore} disabled={busy}>
              Restore
            </button>
          </div>

          {statusMessage && <div className="status-success">{statusMessage}</div>}
          {errorMessage && <div className="status-error">{errorMessage}</div>}
        </div>
      </div>
    </div>
  );
}
