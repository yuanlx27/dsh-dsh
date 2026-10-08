# T029: retained primary window and native navigation

WindowState joins creation, preserves one identity on hide/show and ignores late
old open/failure/destruction. Native Primary schedules on the UI thread, hides
and prevents close destruction, focuses/unminimizes the existing window, and
recreates only absent/failed documents. Replacement detaches before destruction;
real Destroyed invokes the native document hook, not a renderer unload callback.
Single-instance arguments/cwd are ignored; the existing process merely opens its
primary. Dock Reopen uses the same open handler. No runtime start/stop or task
operation is in the window layer. T031 installs these entry hooks.

Current-display work areas validate restored logical bounds; invalid/unavailable
bounds fall back to a centered window. Close saves only bounds through the
Application preference lock, preserving notice and avoiding competing ack writes.
Native single-instance/close/Dock/focus/draft/scroll preservation is not yet
executed as installed WK acceptance.

The WK navigation delegate observer now inspects actual navigation type. Only
owned app/shell URLs stay embedded; file/data/javascript/unapproved authorities
and credential-bearing URLs are denied. Actual LinkActivated HTTP(S) navigation
is cancelled in WK and opened by native NSWorkspace with no service credential.
Other remote/programmatic navigation is denied. Remaining optional callbacks
forward to pinned Wry's original delegate. AppKit is a narrow NSWorkspace-only
promotion of the already-locked 0.3.2 dependency, not an SDK version update.

Native-feature library: 28 tests pass (two added window/navigation tests).
Desktop-feature check and Clippy -D warnings pass. Existing T018 window cases
remain waiting on the T030 quit module for full lifecycle compilation. Actual
WK LinkActivated gesture classification, new-window target handling, delegate
forwarding, failure/Dock events and retained upload/download/document state are
T031/T032 acceptance gates; compiled native handlers are not executed evidence.
