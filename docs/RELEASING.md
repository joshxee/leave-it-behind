# Releasing

`.github/workflows/release.yaml` runs on `v*` tags (or manually with a `tag`
input for re-releases and rollbacks):

1. `check-tag`: the tag starts with `v` and its commit is on `main`.
2. `ci`: the full CI workflow (`ci.yaml`) on the tagged commit. A red commit never ships.
3. Builds: `html5` (via `scripts/build-web.sh --release`: size budget + leak
   check), `windows`, `macOS-intel`, `macOS-apple-silicon`.
4. `release-smoke`: downloads the exact `html5` zip, serves it, and runs the
   `@release-smoke` Playwright spec.
5. `github-release` (the only job with `contents: write`) attaches the builds to the GitHub release.
6. `upload-to-itch`: runs in the protected `release` environment (needs
   approval), and pushes each artifact to the itch channel of the same name
   (`html5`, `windows`, `macOS-intel`, `macOS-apple-silicon`).

macOS runners use about 10× the Actions minutes on private repos. Drop the
`release-macOS` matrix entries if minutes get tight.

## One-time setup (manual)

1. **Workflow permissions:** leave Settings → Actions → General → Workflow
   permissions on the default **Read repository contents** (read-only). The
   workflows request more job by job.
2. **`release` environment:** Settings → Environments → New environment
   `release`. Add yourself as a **required reviewer**. Under deployment
   branches and tags, choose **Selected branches and tags** and add the tag
   rule `v*`.
3. **itch.io API key:** create one at <https://itch.io/user/settings/api-keys>
   and store it as an **environment** secret (not a repository secret):
   `gh secret set BUTLER_CREDENTIALS --env release`. If you created a
   repository secret with the same name earlier, delete it:
   `gh secret delete BUTLER_CREDENTIALS`.
4. **After the first push to itch:** on the `knockbox/leave-it-behind`
   project page, set *Kind of project* to **HTML**. Mark the `html5` upload
   **This file will be played in the browser**, set the viewport to
   **1280 × 720**, and enable the fullscreen button if you want it.

## Cutting a release

```bash
git checkout main && git pull
git tag -a v0.1.0 -m "v0.1.0: first playable"
git push origin v0.1.0
```

Then approve the `release` deployment in the Actions tab (the run pauses at `upload-to-itch`).

## Rollback

```bash
gh workflow run release.yaml -f tag=<previous-good-tag>
```

Approve the deployment. Butler pushes the older build to the same channels.
Confirm the version in the game's bottom-right corner shows the old tag.

## Checking a live release

- Open the itch page. The corner text should show the tag you released.
- Open the browser console. There should be no errors.
- When a player reports a crash, ask for a screenshot of the crash overlay.
  It shows the version (`The game crashed. Version vX.Y.Z…`), which maps to a tag.
