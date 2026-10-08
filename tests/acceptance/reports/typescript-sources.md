# TypeScript-only source migration

## Agreed boundary

Only TypeScript is committed for our JS/TS-layer source, including tooling and
tests. Native Rust, semantic HTML/CSS, configuration and inherited Spec Kit shell
scripts remain. Downloaded dsh/dependencies and compiled browser JavaScript are
Git-ignored, not converted/reimplemented. References to their `.js` paths are
permitted. Existing historical commits are not rewritten.

## Changes

- Migrated all ten tracked `.mjs` files to `.ts`, with TS import paths and updated
  npm/specification references. Node 24.21.0 directly executes erasable TS.
- Added strict, no-emit `tsconfig.tools.json`: explicit Node/file/hash/manifest/
  fixture types, narrowed caught errors and typed process environments. Existing
  runtime checks remain; JSON type annotations are not runtime schema validation.
- Added exact `@types/node@24.19.1` in the development lock, leaving all runtime R
  versions and the frozen production closure unchanged.
- Moved the inline native-addon JS program to a standalone TS fixture executed
  by the actual bundled Node; sharp/koffi/node-pty/flock behavior still passes.
- Added `AGENTS.md`, `check:source`, frontend pre-build checks and a pinned-action
  GitHub workflow. The checker rejects tracked/staged or unignored untracked
  `.js`, `.mjs`, `.cjs`, `.jsx` files (case-insensitive). It does not hide source
  mistakes with global JS ignore patterns.
- The unfinished `transport-adapter.test.ts` remains in the full test command.
  It has an explicit temporary tooling type-check exclusion because T027's module
  does not exist; remove it at T027. No placeholder declaration or test skip was
  added. Do not interpret the implemented-tooling check as complete app typing.

## Actual verification

Before migration, the source-policy assertion failed and listed the ten tracked
JS files. After staging migration, `git ls-files '*.js' '*.mjs' '*.cjs' '*.jsx'`
returned no paths, and both source-policy tests passed.

Executed under Node 24.21.0:

- `npm run check:source`: **2 passing**.
- `npm run typecheck:tools`: **passing** for implemented scripts/tests/fixtures.
- `npm run test:packaging`: **39 passing, 1 failing**; the only failure is the
  previously missing T027 `src/transport.ts`. Migrated 37 regression tests and
  native-addon loading/flock pass, plus the two new source-policy tests.
- `npm run runtime:verify`: **passing** against actual staged R resources.
- Selected Rust library/auth regression command: **20 passing**, plus the
  separate-process proxy check; native code was not changed by this migration.
- `git diff --cached --check`: **passing**.

Initial strict typing surfaced implicit parameters, unknown errors and fixture
lookups; these were fixed rather than suppressing checking. No fresh runtime
preparation/download/signing was necessary for this source-only migration.
The GitHub workflow is defined and its steps tested locally, not observed on a
GitHub runner. Installed WKWebView/app acceptance and other pending tasks remain
pending. Historical reports using `.mjs` commands describe their original runs;
current commands/file paths now use `.ts`.
