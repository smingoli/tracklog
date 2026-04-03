import { useEffect } from "react";
import { NavLink, Route, Routes } from "react-router-dom";
import { getVersion } from "@tauri-apps/api/app";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { HomePage } from "./pages/HomePage";
import { TracksPage } from "./pages/TracksPage";
import { TrackDetailPage } from "./pages/TrackDetailPage";
import { ReleasesPage } from "./pages/ReleasesPage";
import { ReleaseDetailPage } from "./pages/ReleaseDetailPage";
import { SettingsPage } from "./pages/SettingsPage";

export default function App() {
  useEffect(() => {
    (async () => {
      try {
        const version = await getVersion();
        await getCurrentWindow().setTitle(`TrackLog - V: ${version}`);
      } catch (err) {
        console.error("Could not set window title:", err);
      }
    })();
  }, []);

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <h1>TrackLog</h1>
        <nav>
          <NavLink to="/">Home</NavLink>
          <NavLink to="/tracks">Tracks</NavLink>
          <NavLink to="/releases">Releases</NavLink>
          <NavLink to="/settings">Settings</NavLink>
        </nav>
      </aside>

      <main className="content">
        <Routes>
          <Route path="/" element={<HomePage />} />
          <Route path="/tracks" element={<TracksPage />} />
          <Route path="/tracks/new" element={<TrackDetailPage mode="create" />} />
          <Route path="/tracks/:id" element={<TrackDetailPage mode="edit" />} />
          <Route path="/releases" element={<ReleasesPage />} />
          <Route path="/releases/new" element={<ReleaseDetailPage mode="create" />} />
          <Route path="/releases/:id" element={<ReleaseDetailPage mode="edit" />} />
          <Route path="/settings" element={<SettingsPage />} />
        </Routes>
      </main>
    </div>
  );
}