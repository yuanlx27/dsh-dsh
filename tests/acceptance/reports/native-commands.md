# T030: native commands and joined conservative quit

One closed command table drives native clicks/accelerators for Close Window
(Command+W), Quit (Command+Q), Open Workspace Window, About and Safety Notice.
Close uses Primary's retained hide handler. Edit menu keeps standard native
text editing. About shows exact verified desktop/dsh prerelease versions, not
Apple's numeric marketing/build representation. Safety Notice is informational;
only T028's saved current revision can grant entry.

QuitDecision joins one pending native decision, ignores old/disposed replies,
never inspects task/idle state, and requires successful owned cleanup before Exit.
Starting/ready/stopping and unproven cleanup all count as alive. Stay/dismissal/
failure preserve the service; cleanup failure permits only a new explicit choice.
A native ownerless NSAlert explains all workspaces/sessions, unknown work state,
interruption and no undo; first/default Return is Stay, destructive second button
has no Return/Escape key equivalent. Native repeated requests focus the same
alert without showing a hidden WK window. No renderer acknowledgement is used.
Confirmed stop saves bounds, invalidates native authority, awaits owned Runtime
shutdown and only then exits. No-service quit disposes before exit so queued
startup cannot later launch. T031 connects menu/run-event entry dispatch.

All ten previously red T018 lifecycle tests pass. Full default Rust suite: 69
passing plus auth proxy subprocess, no failures. Desktop library check and Clippy
-D warnings pass. Controlled cleanup-result bypass fails the expected no-Exit
assertion; restoring returns green. NSAlert/Menu code compiles against the exact
locked SDK, but actual ownerless dialog/focus, Escape/Return, menu/key equivalence
with Harness focus, repeated hidden/active/pending/unknown quit and installed
cleanup are NOT executed. These remain native T032/US4/VoiceOver gates, not
qualified by these policy fixtures or compilation.
