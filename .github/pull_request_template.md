## What and why

<!-- What was wrong or missing, what is true now, and why this approach. -->

## Checklist

- [ ] The title is a conventional commit (`feat:`, `fix:`, `perf:`, `refactor:`, `docs:`, `chore:`, `test:`, `ci:`). It becomes the squash commit and the changelog line, so write it for the person reading the changelog.
- [ ] A branch with a single commit: that commit's subject says the same thing as the title. The squash commit takes the title, but the two should never disagree.
- [ ] No `!` after the type and no `BREAKING CHANGE:` footer before 1.0.0. Releases before 1.0 are one minor version per phase (see `docs/releases.md`).
- [ ] `cargo fmt`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass locally.
- [ ] Docs and README updated if behaviour, keys or options changed. English only.
