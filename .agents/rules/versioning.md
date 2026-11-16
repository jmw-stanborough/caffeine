---
trigger: always_on
---

# Automatic Patch Version Increments

- Whenever a task modifies application source code (`src/` or `src-tauri/`), run `pnpm version:patch` before making the final commit so the patch version (`Z` in `X.Y.Z`) is incremented across `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, and the in-app sidebar badge.
- Never bump the minor (`Y`) or major (`X`) version (`pnpm version:set`) and never create or push release tags (`pnpm release:tag`) unless the user explicitly asks for a minor/major version change or a release.
