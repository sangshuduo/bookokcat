# Repository Guidelines

## Project Structure & Module Organization
- `src/` hosts the application crate; higher-level UI lives in `src/widget/`, rendering helpers in `src/components/`, parsing logic in `src/parsing/`, and vendored rendering backends in `src/vendored/`.
- `tests/` contains async integration and snapshot suites (`snapshots/`, `svg_snapshots/`, `testdata/`), while `src/test_utils/` exposes builders for simulated key input.
- `docs/` captures design notes and walkthroughs; `examples/` and `temp_images/` hold reference assets used by tests and docs.
- Runtime artifacts such as `bookokcat.log` and flamegraphs land in the repository root by default—clean them up before committing.

## Build, Test, and Development Commands
- `cargo fmt` formats the Rust sources using the edition 2024 defaults.
- `cargo clippy --all-targets --all-features -D warnings` keeps the codebase lint-clean; run this before opening a PR.
- `cargo build` and `cargo run -- <path-to-book>` compile the TUI and run it against a local EPUB directory.
- `cargo test` executes unit and snapshot suites; add `-- --ignored` to include slow image-diff checks, and `cargo test visual_diff` to focus on graphical regressions.

## Coding Style & Naming Conventions
- Stick to Rust 2024 idioms: four-space indentation, `snake_case` for functions and modules, `UpperCamelCase` for types, and `SCREAMING_SNAKE_CASE` for constants.
- Prefer small, composable modules; align new widgets with the patterns in `src/widget/navigation_panel/` or `src/widget/text_reader/`.
- Keep imports sorted, avoid wildcard globbing outside tests, and document non-obvious state transitions with concise `///` doc comments.

## Testing Guidelines
- Snapshot tests rely on `snapbox` and SVG fixtures; when snapshots change legitimately, run `cargo insta review` and check files under `tests/snapshots/` and `tests/svg_snapshots/`.
- Name new tests after the user interaction they cover (e.g., `mouse_scroll_flood_test` style) and group shared helpers under `src/test_utils/`.
- Maintain deterministic inputs: reuse assets in `tests/testdata/` and avoid relying on system clocks or network access.

## Commit & Pull Request Guidelines
- Follow the existing history: short, imperative summaries such as “Fix mouse scroll flood” or “Tighten clippy linting”; include scope prefixes only when they aid clarity.
- Each PR should link to any tracked issue, describe behavioral changes, and include before/after terminal captures or SVG diffs when altering rendering.
- Confirm CI prerequisites locally (`cargo fmt`, `cargo clippy`, `cargo test`) and note any ignored tests or platform caveats in the PR description.
