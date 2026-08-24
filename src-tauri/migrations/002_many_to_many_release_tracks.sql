CREATE TABLE release_tracks_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    release_id INTEGER NOT NULL,
    track_id INTEGER NOT NULL,
    track_order INTEGER NOT NULL CHECK (track_order > 0),
    created_at TEXT NOT NULL,
    FOREIGN KEY (release_id) REFERENCES releases(id) ON DELETE CASCADE,
    FOREIGN KEY (track_id) REFERENCES tracks(id) ON DELETE CASCADE,
    UNIQUE (release_id, track_id),
    UNIQUE (release_id, track_order)
);

INSERT INTO release_tracks_new (id, release_id, track_id, track_order, created_at)
SELECT id, release_id, track_id, track_order, created_at
FROM release_tracks;

DROP TABLE release_tracks;
ALTER TABLE release_tracks_new RENAME TO release_tracks;

CREATE INDEX idx_release_tracks_release_id ON release_tracks(release_id);
CREATE INDEX idx_release_tracks_track_order ON release_tracks(release_id, track_order);
CREATE INDEX idx_release_tracks_track_id ON release_tracks(track_id);
