PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    applied_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tracks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    internal_code TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('Idea', 'Draft', 'In Progress', 'Final')),
    description TEXT,
    lyrics TEXT,
    notes TEXT,
    bpm INTEGER CHECK (bpm IS NULL OR bpm > 0),
    musical_key TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS releases (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    internal_code TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    type TEXT NOT NULL CHECK (type IN ('Album', 'EP', 'Single')),
    status TEXT NOT NULL CHECK (status IN ('Planned', 'In Progress', 'Released')),
    description TEXT,
    image_path TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS release_tracks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    release_id INTEGER NOT NULL,
    track_id INTEGER NOT NULL UNIQUE,
    track_order INTEGER NOT NULL CHECK (track_order > 0),
    created_at TEXT NOT NULL,
    FOREIGN KEY (release_id) REFERENCES releases(id) ON DELETE CASCADE,
    FOREIGN KEY (track_id) REFERENCES tracks(id) ON DELETE CASCADE,
    UNIQUE (release_id, track_order)
);

CREATE INDEX IF NOT EXISTS idx_tracks_title ON tracks(title);
CREATE INDEX IF NOT EXISTS idx_tracks_status ON tracks(status);
CREATE INDEX IF NOT EXISTS idx_tracks_updated_at ON tracks(updated_at);

CREATE INDEX IF NOT EXISTS idx_releases_title ON releases(title);
CREATE INDEX IF NOT EXISTS idx_releases_type ON releases(type);
CREATE INDEX IF NOT EXISTS idx_releases_status ON releases(status);
CREATE INDEX IF NOT EXISTS idx_releases_updated_at ON releases(updated_at);

CREATE INDEX IF NOT EXISTS idx_release_tracks_release_id ON release_tracks(release_id);
CREATE INDEX IF NOT EXISTS idx_release_tracks_track_order ON release_tracks(release_id, track_order);

