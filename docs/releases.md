# Releases

Collector is released the way every Project Colony program is: by
release-please and the shared signing workflow of
[Project-Colony-Resources](https://github.com/Project-Colony/Project-Colony-Resources)
(its `design/releases.md` is the reference). Nothing is released by hand.

## The flow

1. Pull requests are squash-merged into `main`. The squash commit takes the
   pull request title (repository setting: default to the pull request
   title), a conventional commit (`feat:`, `fix:`, `docs:` and so on), which
   becomes the changelog line. On a single-commit branch, keep the commit's
   subject the same as the title anyway.
2. On every push to `main`, release-please (`.github/workflows/release.yml`,
   configured by `release-please-config.json` and
   `.release-please-manifest.json`) opens or updates a release pull request.
   It bumps the version in `Cargo.toml` and `Cargo.lock` and writes
   `CHANGELOG.md`. Only `feat`, `fix`, `perf`, `refactor` and `docs` commits
   make a release; `chore`, `test` and `ci` are hidden. It opens that pull
   request with the workflow's own token, which GitHub refuses unless
   **Settings > Actions > General > Workflow permissions > Allow GitHub
   Actions to create and approve pull requests** is ticked; if it was not
   when a push reached `main`, tick it and re-run the failed job. The default
   permissions stay read-only: each workflow declares what it needs.
3. Merging the release pull request creates the tag and the GitHub release.
   The release is immediately turned back into a **draft**, because
   release-please publishes it before any binary exists, and every Colony in
   the field would otherwise offer an update with nothing to download.
4. Four build legs check out the tag and build with `--locked` and no cache:
   `collector-linux`, `collector-windows.exe`, `collector-macos` (Apple
   Silicon) and `collector-macos-x86` (Intel). Each is smoke-tested with
   `--version` (the Intel build, cross-compiled, has its architecture checked
   instead), the Windows build has its version resource checked, and
   `colony.json` is validated against the asset names.
5. The shared `sign-and-publish` workflow signs and publishes, in the same
   run: Authenticode through SignPath for the Windows file once SignPath has
   approved the project (the slug is commented out in `release.yml` until
   then), then the ed25519 `.sig`, `.meta` and `.meta.sig` for every asset
   with the organisation's key, an upload to the draft, a verification of what
   was uploaded, and finally publishing.

Merging the release pull request is therefore the decision to publish: once
every check above passes, the release goes public with no further click.

If a run fails after the tag exists, the release stays a draft. Re-running
the original run does nothing (release-please reports the release only once);
finish it with a dispatch started from the tag:

```bash
gh workflow run release.yml -R Project-Colony/Collector --ref v0.2.0 -f tag=v0.2.0
```

## Versions before 1.0

Each step of the [roadmap](../README.md#roadmap) is one minor release. The
release pull request is merged only when that step is done; until then it
keeps collecting commits.

`release-please-config.json` is the organisation template unchanged, so
`"bump-minor-pre-major"` is `false`: a single `feat!:` commit or
`BREAKING CHANGE:` footer turns the next release into 1.0.0. The rewrite
changes the command line, the keys and the configuration, exactly the commits
someone would mark that way, hence the rule: no `!` and no `BREAKING CHANGE:`
footer before 1.0. Describe the break in the commit body instead. 1.0.0 itself
is requested with a `Release-As: 1.0.0` footer.

The manifest starts at 0.1.0, the version that predates this pipeline, and no
release or tag for it exists. release-please then counts every commit on
`main` and bumps from 0.1.0: a `feat` commit makes the first release 0.2.0. Its
`initial-version` option would not help here: the `rust` release type ignores
it.
