<div align="center">

# DSH²

DeepSeek Harness Desktop SHell.

![Vibe Coding Badge](.github/assets/badge-vibe-coding.svg) ![SDD Badge](.github/assets/badge-sdd.svg)

</div>

## Status

Experimental macOS Tauri shell for bundled **dsh 0.2.0-rc.2** and
**Node 24.21.0**. The original upstream Harness owns models, credentials,
workspaces, tasks, approvals and history. Native transport, window lifecycle
and explicit recovery are implemented and have scoped installed-app evidence;
full acceptance and local distribution packaging are not complete.

Target: macOS 14+ Apple Silicon. Installed candidates have been tested on
macOS 27.0.1; macOS 14, clean-account and final DMG qualification remain pending.
Local candidates use ad-hoc signing, not Developer ID/notarization/Gatekeeper
qualification. See [task status](specs/001-deepseek-harness-desktop/tasks.md) and
[acceptance reports](tests/acceptance/reports/).

## Development

Build host needs Node 24.21.0/npm, the pinned Rust toolchain, Xcode Command Line
Tools and registry access for locked build-time downloads. Installed users do
not need separate Node/npm/dsh; tools used by approved project actions are
separate prerequisites. All own JS/TS-layer source is TypeScript.

```sh
npm ci
npm run check:source
npm run typecheck:tools
npm run runtime:prepare
npm run runtime:verify
cargo test --manifest-path src-tauri/Cargo.toml
npm run test:packaging
npm run tauri:dev
npm run tauri:build -- --bundles app
```

The Tauri build creates a **build candidate**, not a qualified installable
release. Final staging/symlink restoration, nested signing, post-sign inventory,
outer signing and .app/DMG packaging must follow T052; its maintained packaging
command remains unfinished. Do not modify resources after outer signing or
install a raw candidate as if final integrity had been verified. Acceptance-only
candidate procedures/results are documented in the reports; see
[quickstart](specs/001-deepseek-harness-desktop/quickstart.md) for qualification.

## Lifecycle and safety

- Close Window / Command+W hides and retains the workspace and owned service;
  reopening does not intentionally reset drafts or stop tasks.
- Quit / Command+Q asks Stay or Stop and Quit while service ownership may be live.
  Stay does not request shutdown; confirmed quit waits for owned cleanup and
  does not undo completed actions, across all sessions/workspaces.
- Backend failure shows a non-secret local explanation. Explicit Retry starts
  fresh owned/authenticated state without resubmitting inputs. Review upstream
  interrupted history; unsaved RAM progress is not guaranteed after a crash.
- Harness is experimental and can run commands/change files. Review permissions
  and backups; prompts are not a sandbox or rollback guarantee.
- Upstream model credentials are plaintext with owner-only permissions. Same-user
  processes, including approved tools, can read them. Local bearer authentication
  is not process-identity isolation against stolen/forged authentication.
- Service tokens/cookies stay native; the shell does not promise blanket redaction
  of arbitrary upstream/tool/model output or copy credentials into a second store.

Use disposable projects and public synthetic fixture keys for tests. Never
commit secrets, token-bearing URLs or actual project/session data.
