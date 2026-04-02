# TrackLog

TrackLog is a local-first desktop application for managing music tracks and releases.

It is designed for musicians, bands, producers, songwriters, and anyone who creates music. As a lightweight catalog tool, it helps organize song ideas, drafts, finished tracks, and release groupings such as albums, EPs, and singles.

The application runs as a native desktop app with a React frontend, a Rust/Tauri backend, and a local SQLite database.

---

## What TrackLog Does

TrackLog currently supports:

- creating, editing, and deleting tracks
- creating, editing, and deleting releases
- assigning a track to a release
- removing a track from a release
- ordering tracks inside a release
- filtering tracks by status and availability
- filtering releases by type and status
- searching tracks by internal code or title
- uploading and managing release artwork locally
- showing dashboard summaries and recently updated items

Tracks and releases are managed locally on the user’s machine. The current implementation is focused on speed, simplicity, and a clear desktop workflow rather than cloud sync or collaboration.

---

## Tech Stack

### Frontend
- React 18
- TypeScript
- React Router
- Vite

### Desktop Runtime
- Tauri 2
- Tauri Dialog plugin

### Backend
- Rust
- SQLite via `rusqlite`
- `serde`, `chrono`, `dirs`

Dependencies and build scripts are defined in `package.json` and `src-tauri/Cargo.toml`.

---

## Application Structure

```text
tracklog/
├── src/                    # React frontend
│   ├── pages/              # Main application pages
│   ├── services/           # Tauri invoke wrappers
│   └── types/              # Shared frontend models
├── src-tauri/
│   ├── src/
│   │   ├── commands.rs     # Tauri command layer
│   │   ├── db/             # Database logic and business rules
│   │   ├── fs.rs           # Storage path and file helpers
│   │   ├── lib.rs          # Tauri bootstrap and command registration
│   │   └── models.rs       # Backend serialization models
│   └── migrations/         # SQLite schema migrations
├── package.json
└── README.md
```

The frontend talks to the backend through Tauri commands exposed in Rust and wrapped on the frontend in `src/services/tauri.ts`.

---

## Main Screens

### Home
The home page shows:
- total tracks
- available tracks
- total releases
- recently updated tracks
- recently updated releases

It also provides quick actions to create a new track or a new release.

### Tracks
The tracks page provides:
- searchable track list
- status filter
- availability filter
- direct navigation to track detail
- quick action to create a new track

Track search is performed by internal code or title.

### Track Detail
Track detail supports both create and edit modes.

Track fields include:
- internal code
- title
- status
- BPM
- key
- description
- lyrics
- notes

In create mode, TrackLog also supports **Save and Add Another** for faster consecutive data entry. In edit mode, a track can be assigned to a release or removed from its current release.

### Releases
The releases page provides:
- release list
- type filter
- status filter
- track counts
- quick action to create a new release

Supported release types:
- Album
- EP
- Single

Supported release statuses:
- Planned
- In Progress
- Released

### Release Detail
Release detail supports both create and edit modes.

Release fields include:
- internal code
- title
- type
- status
- description
- image

In edit mode, users can:
- upload or replace release artwork
- remove release artwork
- add available tracks to the release
- remove tracks from the release
- reorder tracks using Move Up / Move Down actions

Artwork is previewed through Tauri file handling and stored in TrackLog-managed local storage.

---

## Data Model

### Track
A track is a standalone catalog item representing a song idea, draft, or completed track.

Supported track statuses:
- Idea
- Draft
- In Progress
- Final

### Release
A release is a grouping of tracks such as an album, EP, or single.

Each release can optionally store a managed local artwork path.

### Release Assignment
A track can be assigned to only one release at a time. This is enforced in the database schema and backend logic. Track ordering inside a release is stored explicitly and can be changed later.

---

## Local Storage

TrackLog stores application data locally.

### Database
The SQLite database is stored under the application data directory as:

```text
TrackLog/data/catalog.db
```

### Release artwork
Managed release images are stored under:

```text
TrackLog/data/images/releases/
```

The backend creates these directories automatically when needed. File names are sanitized and derived from the release internal code.

On Windows, this is typically under the user’s Local AppData directory.

---

## Database Schema

The current SQLite schema includes three main tables:

- `tracks`
- `releases`
- `release_tracks`

Important rules currently enforced:
- track internal code must be unique
- release internal code must be unique
- track status must be one of the supported values
- release type and status must be valid
- BPM must be positive when provided
- a track may belong to only one release at a time
- track order must be unique within a release

The initial schema is defined in `src-tauri/migrations/001_initial_schema.sql`.

---

## Backend Commands

The frontend uses Tauri commands for all data access and mutations. Current commands include operations such as:

- initialize app storage and schema
- list and search tracks
- create, update, and delete tracks
- list available tracks
- list releases
- create, update, and delete releases
- assign or remove tracks from releases
- move tracks within a release
- set or remove release images
- get dashboard summary

The command registration is defined in `src-tauri/src/lib.rs`, with thin forwarding logic in `src-tauri/src/commands.rs`.

---

## Requirements

For local development on Windows, install:

- Node.js LTS
- Rust via `rustup`
- Visual Studio Build Tools 2022
- WebView2 Runtime if it is not already present

---

## Setup

Clone the repository and install JavaScript dependencies:

```bash
git clone https://github.com/smingoli/tracklog.git
cd tracklog
npm install
```

Rust dependencies are resolved automatically by Cargo during the first Tauri build.

---

## Development

Run the application in development mode:

```bash
npm run tauri dev
```

This starts the Vite frontend and the Tauri desktop shell together. The frontend entry point is `src/main.tsx`.

---

## Build

Create a production build:

```bash
npm run tauri build
```

Typical build output locations:

```text
src-tauri/target/release/
src-tauri/target/release/bundle/
```

---

## Current Status

TrackLog is already a functional local desktop catalog manager, not just a scaffold. The current codebase includes:

- working track CRUD
- working release CRUD
- release assignment logic
- release track ordering
- local SQLite persistence
- local release artwork management
- dashboard summaries

That said, it is still early-stage and intentionally focused.

---

## Current Limitations

The current version does not yet include:

- cloud sync
- multi-user collaboration
- import/export tools
- advanced full-text search across all fields
- automated tests
- backup/restore workflows
- bulk editing
- tagging or custom metadata fields

---

## Roadmap Ideas

Possible next steps for the project:

- richer search and filtering
- backup/export support
- import tools for existing catalogs
- better validation and inline error handling
- track tagging and categorization
- richer artwork handling
- test coverage for backend logic
- packaging and release automation

---

## Author

Stefano Mingoli

---

## License

No license has been defined yet.

Until a license is added, this repository should be treated as all rights reserved by default.
