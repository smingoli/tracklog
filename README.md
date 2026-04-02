# SongsCatalog V1 Core Complete Baseline

This package extends the previous baseline and aims to cover the full agreed **v1 core**.

## Implemented in this package
- Local AppData storage init
- migration-aware SQLite startup
- dashboard summary queries
- Track CRUD
- Release CRUD
- assign track to release
- remove track from release
- list only available tracks for assignment
- reorder tracks inside a release
- continuous track numbering
- release image set / replace / remove
- managed image storage under:
  - `%LOCALAPPDATA%\SongsCatalog\data\images\releases\`

## Important notes
- This is still a code baseline, not a compiled Windows installer.
- The code is structured to match the agreed requirements and should be used as the next implementation base.
- Image picking uses the Tauri dialog plugin from the frontend.

## Expected storage paths
- `%LOCALAPPDATA%\SongsCatalog\data\catalog.db`
- `%LOCALAPPDATA%\SongsCatalog\data\images\releases\`
