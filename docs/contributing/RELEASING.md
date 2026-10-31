# Releasing

The workspace version (`[workspace.package] version`) is the application version. 0.1.0 is the first release.

1. **Gate.** `scripts/check.sh` passes on a clean checkout of `main`; `git status` is clean.
2. **Heavy tests.** With the language model installed, run the release-only tests that the gate skips:
   `cargo test --release -p studyfrog-candle -- --nocapture` (latency budget, calibration).
3. **Benchmarks.** `cargo bench --workspace` on an otherwise idle machine; compare with
   [BENCHMARKING](BENCHMARKING.md) and record any change in the baselines table.
4. **Changelog.** Rename `## [Unreleased]` in `CHANGELOG.md` to `## [x.y.z] - YYYY-MM-DD` and add a fresh empty
   `## [Unreleased]` above it. Update the release notes in `docs/RELEASE_NOTES_x.y.z.md`.
5. **Version.** For anything after 0.1.0 bump `[workspace.package] version`, run `cargo check` to refresh
   `Cargo.lock`, commit as `chore(release): x.y.z`.
6. **Package.** `scripts/package-appimage.sh` (downloads a pinned `appimagetool`); start the resulting
   `target/appimage/StudyFrog-x.y.z-x86_64.AppImage` on a desktop session and open a vault. Record its
   SHA-256 (`sha256sum`) next to the file. (0.1.0 was packed and started this way on 2026-10-30.)
7. **Tag.** (The `release` workflow in `.github/workflows/release.yml` does the publishing below when a `v*` tag is pushed:
   it runs the gate, builds the AppImage and creates the GitHub release from `docs/RELEASE_NOTES_x.y.z.md`.) `git tag -a vx.y.z -m "StudyFrog x.y.z"`. Push the tag, and attach the AppImage and its checksum to
   the release, once a remote exists (CI in `.github/workflows/ci.yml` builds the same image on every push to
   `main` as an artifact).

Nothing in this list is automated beyond steps 1 and 6 in CI. A release is only claimed after step 6 has been
done on a real desktop.
