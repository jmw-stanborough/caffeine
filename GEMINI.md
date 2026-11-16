# Workspace Rules

## Application Versioning

- Whenever a task modifies application source code (`src/` or `src-tauri/`), run `pnpm version:patch` before making the final commit so the patch version (`Z` in `X.Y.Z`) is incremented across `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, and the in-app sidebar badge.
- When multiple agents or swarm workers operate in parallel branches or worktrees, individual branch workers must not run `pnpm version:patch` (to avoid merge conflicts on version files); instead, the integrator or coordinator runs `pnpm version:patch` once after merging the parallel branches into `main`.
- Never bump the minor (`Y`) or major (`X`) version (`pnpm version:set`) and never create or push release tags (`pnpm release:tag`) unless the user explicitly asks for a minor/major version change or a release.
