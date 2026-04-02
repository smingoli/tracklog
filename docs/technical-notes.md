# Technical Notes

## Current state
This baseline now includes:
- startup initialization
- migration-aware DB bootstrapping
- dashboard summary queries
- Track CRUD
- Release CRUD
- track-to-release assignment
- ordering up/down inside releases
- release image set/replace/remove

## Important notes
- This is the v1 core baseline, but it is still source code and has not been compiled in this environment.
- Image selection uses the Tauri dialog plugin.
- The image preview path may need a small adjustment depending on final Tauri asset loading strategy.

## Suggested next coding order after this baseline
1. compile and fix any environment-specific issues
2. refine notifications and validation messages
3. improve image preview/display strategy
4. optionally add export, tags, or richer dashboard features
