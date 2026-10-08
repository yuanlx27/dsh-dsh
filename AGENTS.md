# Source-language policy

All committed JS/TS-layer source must be TypeScript, including build scripts,
tests and browser/injected programs. Do not add `.js`, `.mjs`, `.cjs` or `.jsx`
files or hide executable JavaScript programs in source strings. Keep compiled
JavaScript, downloaded upstream dsh and third-party dependencies ignored.
References to those generated/third-party paths are permitted. Native Rust,
HTML/CSS, configuration and existing Spec Kit shell tooling remain in scope.

Run `npm run check:source` and `npm run typecheck:tools` before committing.
The source policy also runs before the frontend build and in GitHub Actions.
Node 24.21.0 runs erasable TypeScript directly; this does not replace `tsc`
type checking. Do not silence type failures with `any` or `@ts-nocheck`.
The pending transport contract has an explicit temporary tooling-check exclusion
until its T027 implementation exists; its runtime test must still fail rather
than being skipped. Remove that exclusion when implementing T027.
