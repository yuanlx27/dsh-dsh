# Feature Specification: DeepSeek Harness Desktop

**Feature Branch**: `001-deepseek-harness-desktop`

**Created**: 2026-10-06

**Status**: Draft — validated for planning

**Input**: User description (English translation): "I want to build a desktop application for DeepSeek Harness (https://github.com/deepseek-ai/deepseek-harness)."

**Additional Input** (English translation): "Implementation details should, wherever practical, follow the official desktop implementation: https://github.com/deepseek-ai/deepseek-harness/tree/master/apps." This is a supplement to this feature, not a separate feature.

## Clarifications

### Session 2026-10-06

- Q: What responsibilities belong to upstream dsh and to this desktop product? → A: Upstream dsh is the kernel that starts and provides the Web service; our own desktop shell provides window management, keyboard shortcuts, and other desktop application functions.
- Q: Should dsh and running tasks continue when the user closes the last window? → A: Yes. Closing windows leaves dsh running in the background; only fully quitting the application stops dsh during normal operation.
- Q: Which desktop stack and runtime distribution should the product use? → A: Use Tauri and bundle both Node.js and dsh as application sidecars. The desktop product version must equal the bundled dsh version.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Open a ready-to-use local desktop workspace (Priority: P1)

As an individual developer, I want to open an installed desktop application, configure my model credentials, and choose a local project so I can use DeepSeek Harness without manually starting it in a terminal or managing a browser tab.

**Why this priority**: A desktop application only provides value if users can reach a usable project environment through the application itself.

**Independent Test**: On a supported computer with a valid model credential and an accessible sample project, launch the application, complete setup, and reach a session ready for input without terminal commands.

**Acceptance Scenarios**:

1. **Given** a first launch, **When** the user opens the application, **Then** they see the developer-preview safety notice and a guided path to configure a model and select a workspace.
2. **Given** a valid credential and an accessible project directory, **When** the user saves the configuration and selects that directory, **Then** the application displays the selected workspace and enables task input.
3. **Given** missing or rejected credentials, **When** the user attempts to run a task, **Then** the application explains what to correct, preserves non-secret setup values, and does not present the environment as ready.
4. **Given** previously saved setup and an available workspace, **When** the user relaunches the application, **Then** they can return to that workspace without re-entering credentials.
5. **Given** local Harness operation cannot start, **When** the user launches or retries, **Then** the application displays a failure reason and a retry action rather than an indefinitely blank or loading window.
6. **Given** dsh is running with an active task, **When** the user closes the last window through its close control or Command+W, **Then** the application remains running, dsh continues in the background, and the task is not cancelled by the window closure.
7. **Given** all windows are closed and dsh is still running, **When** the user reopens the application window, **Then** it reconnects to the same dsh instance and shows the current session state without restarting dsh or replaying the task.
8. **Given** dsh is running and the user has confirmed stopping any active work, **When** the user fully quits through the application menu or Command+Q, **Then** the application stops its dsh instance before completing the quit.

---

### User Story 2 - Complete a task with visible progress and control (Priority: P1)

As a developer, I want to ask the agent to work on my selected project, follow its responses and actions, and approve or stop work so I retain control over changes to my computer.

**Why this priority**: Completing a project task is the central reason to use Harness; visibility and permission decisions are necessary for responsible use.

**Independent Test**: With a prepared workspace and model configuration, ask the agent to summarize a sample project and propose a small file change. Verify progress, required permission decisions, final results, and cancellation using controlled tasks.

**Acceptance Scenarios**:

1. **Given** a ready workspace, **When** the user creates a session and submits a task, **Then** their request appears once and the session displays running status, responses, reported actions, and the final outcome.
2. **Given** an action requires approval under the active permission policy, **When** approval is requested, **Then** the user sees the proposed action and its target and can allow or deny it before it proceeds.
3. **Given** a pending permission request, **When** the user denies it, **Then** that action is not performed and the session visibly records the denial.
4. **Given** an active task, **When** the user selects Stop, **Then** the application immediately acknowledges the request, stops initiating further actions for that task, and distinguishes completed actions from any action still finishing.
5. **Given** a service failure during a task, **When** work cannot continue, **Then** the session shows failed or interrupted status, preserves the displayed conversation, and offers an explicit recovery action without automatically repeating file changes or commands.
6. **Given** the user attempts to change the active workspace or quit the application during running work, **When** the application receives that request, **Then** it warns about the active task and allows the user to remain or explicitly stop and leave.

---

### User Story 3 - Return to previous project conversations (Priority: P2)

As a developer, I want to reopen earlier sessions in their original workspaces and continue the conversation so I do not lose project context between desktop launches.

**Why this priority**: Durable sessions support ongoing development without introducing collaboration or cloud synchronization.

**Independent Test**: Prepare completed and interrupted sessions in two workspaces, close and reopen the application, and verify that the user can find each session, view its history, and send a follow-up in the correct workspace.

**Acceptance Scenarios**:

1. **Given** a completed session, **When** the user reopens the application and selects that session, **Then** its saved messages, reported actions, permission decisions, and outcome are displayed under its original workspace.
2. **Given** a restored session with an available workspace, **When** the user sends a follow-up, **Then** work continues in the same session and workspace without changing another project's history.
3. **Given** an unexpected application exit during work, **When** the user reopens the session, **Then** persisted history remains readable, the incomplete task is labeled interrupted, and no pending permission is treated as approved.
4. **Given** the original workspace was moved or removed, **When** the user opens its session, **Then** history remains readable but further task execution is blocked until the user explicitly selects and confirms a replacement directory.

---

### User Story 4 - Rely on official desktop behavior (Priority: P2)

As a developer, I want familiar desktop actions to follow the official application's behavior wherever compatible with this product's agreed constraints, so that the application behaves predictably without adding unrelated features.

**Why this priority**: Official behavior provides an established reference while our own shell, background lifetime, safety, and release constraints remain binding.

**Independent Test**: Compare the supported desktop behaviors against a recorded official revision and verify that any differences are intentional and explained.

**Acceptance Scenarios**:

1. **Given** planning or review of desktop behavior, **When** launch/readiness, window lifecycle, quit confirmation, menus/shortcuts, error recovery, and release compatibility are examined, **Then** each area identifies the official reference revision and source location, adopted/adapted/excluded decision, expected outcome, and acceptance scenario; differences state their reason and user impact.
2. **Given** official behavior conflicts with an approved requirement or is outside scope, **When** it is considered for adoption, **Then** the approved constraint takes precedence and the difference is documented without implicitly adding capabilities or platforms.
3. **Given** an available menu action and its keyboard shortcut, **When** either route is used, **Then** both produce the same outcome while retaining the distinction between closing windows and fully quitting.
4. **Given** a quit confirmation is unresolved or running work cannot be determined, **When** quit is requested again or with uncertain work status, **Then** only one unresolved confirmation is presented and possible interruption is warned about rather than silently quitting; cancellation preserves work.
5. **Given** the official reference or supported Harness release changes, **When** adoption is proposed, **Then** affected decisions and comparison scenarios are reviewed before changing the baseline; unavailable evidence remains pending rather than being reported as passing.

### Edge Cases

- A workspace is unreadable or becomes unavailable: explain the access problem, retain history, and block new work against that directory.
- The model service is unreachable, rejects credentials, or limits usage: show the cause when known and an explicit retry action; do not silently resubmit tasks.
- A user double-submits while a task is starting: create only one active task for that submission and indicate that work has started.
- A Stop request arrives while a file change or command is already underway: acknowledge the request and report the action's eventual outcome; do not imply completed changes were undone.
- The application exits with an unresolved permission request: reopening must not grant that permission or automatically execute the action.
- The last window closes while a task runs or approval is pending: dsh remains running; window closure neither cancels the task nor grants or denies the pending approval. Reopening shows the current task or pending request.
- The user quits while all windows are closed and a task is running: the same active-work confirmation applies as when a window is open; declining keeps the application and dsh running.
- Local Harness operation exits unexpectedly: preserve saved history, show interrupted status, and allow recovery without duplicating the previous task.
- The available Harness version is incompatible with the application: explain the incompatibility and supported version before enabling task execution.
- A session has at least 1,000 messages: it remains navigable and meets the history performance target below.
- The official moving branch differs from the bundled release, or a behavior exists only on another platform: assess compatibility and document adaptation or exclusion without expanding scope.
- Official source or comparison evidence is unavailable: retain the last recorded baseline, mark evidence pending, and do not claim verified parity.
- Official behavior has no counterpart or exposes sensitive configuration: preserve this product's agreed behavior and credential-concealment requirements rather than copying it blindly.
- Repeated quit requests or unknown work status: share one unresolved confirmation and warn about possible interruption.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The product MUST provide an installable macOS desktop application that launches into its own window and makes local Harness operation available without routine terminal commands or a separate browser session. Acceptance: User Story 1, scenarios 1, 2, and 5.
- **FR-002**: The application MUST display setup, ready, starting, or failed status as appropriate and provide a retry action when local operation cannot start. Acceptance: User Story 1, scenarios 2, 3, and 5.
- **FR-003**: Users MUST be able to enter, replace, and remove the credential for at least one supported DeepSeek model connection; removing the credential MUST prevent new tasks that require it. Invalid credentials MUST produce a corrective message rather than a successful setup indication. Acceptance: User Story 1, scenarios 2–4, plus replacement and removal tests.
- **FR-004**: Saved credentials MUST remain concealed in routine settings views and MUST NOT appear in conversation history or application-authored diagnostic messages. The application MUST retain credentials between normal launches until the user removes or replaces them. Acceptance: inspect settings, saved history, and diagnostics with a known test credential before and after relaunch and removal.
- **FR-005**: Users MUST be able to add and select an accessible local directory as a workspace. The active workspace MUST remain visible; task submission MUST be unavailable when no valid workspace is selected. Acceptance: User Story 1, scenario 2, and the unreadable-workspace edge case.
- **FR-006**: Users MUST be able to create a session within the active workspace and submit a natural-language task. Each submitted task MUST be associated with that session and workspace, with at most one task running in a session at a time. Acceptance: User Story 2, scenario 1, and the double-submit edge case.
- **FR-007**: A session MUST display the user's request, agent responses as they become available, reported actions with their outcomes, and a distinguishable idle, running, awaiting-approval, stopping, completed, failed, or interrupted state. Acceptance: User Story 2, scenarios 1–5.
- **FR-008**: Actions requiring approval under the active Harness permission policy MUST remain pending until an explicit user decision. The application MUST show the policy in effect, the requested action and target, and allow/deny controls; denial or dismissal MUST NOT be interpreted as approval. Acceptance: User Story 2, scenarios 2 and 3, and the unresolved-permission edge case.
- **FR-009**: Users MUST be able to request that an active task stop. The application MUST acknowledge the request, prevent new actions for that task, and report any already-started action that cannot immediately stop. It MUST NOT claim that stopping restores changed files. Acceptance: User Story 2, scenario 4, and the in-progress-action edge case.
- **FR-010**: Quitting the application or changing its active workspace during running work MUST require an explicit choice to remain or stop and leave; declining MUST preserve the running session and workspace. Acceptance: User Story 2, scenario 6.
- **FR-011**: The application MUST preserve session history and workspace associations across normal relaunches and MUST let users list, reopen, and continue saved sessions. Acceptance: User Story 3, scenarios 1 and 2.
- **FR-012**: After an unexpected exit, the application MUST retain previously persisted session records, label unresolved work interrupted, and require a new user action before resuming execution. Acceptance: User Story 3, scenario 3.
- **FR-013**: When a session's workspace is unavailable, the application MUST permit history review but block execution until the user explicitly confirms an accessible replacement directory. Acceptance: User Story 3, scenario 4.
- **FR-014**: Startup, model-service, workspace-access, and interrupted-task errors MUST include a user-readable explanation and a next action. Recovery MUST NOT automatically replay commands, edits, or pending approvals. Acceptance: failure scenarios in User Stories 1–3 and the corresponding edge cases.
- **FR-015**: Before the first task, users MUST acknowledge a notice that Harness is experimental, may run commands and modify files, and that permission prompts do not guarantee isolation or undo. The notice MUST remain accessible thereafter. Acceptance: a fresh installation cannot submit its first task before acknowledgement, and a returning user can reopen the notice.
- **FR-016**: The desktop application MUST preserve the chosen supported Harness release's task, workspace, and approval behavior rather than replace it with conflicting behavior. Unsupported versions MUST be identified and blocked before task execution. Acceptance: run the same controlled task and permission scenarios against the supported upstream release and the desktop application; test a known unsupported version.
- **FR-017**: Workspace content and task messages MUST only be sent for user-requested Harness operations to the configured model destination or destinations required by explicitly approved actions under the displayed permission policy. No additional cloud synchronization or product telemetry is in scope. Acceptance: controlled task and idle-launch tests confirm that only the configured destinations and approved actions receive this data.
- **FR-018**: The product MUST use upstream dsh as its kernel and present its Web experience within our own desktop shell. dsh MUST retain responsibility for the Web service, model configuration, workspaces, tasks, approvals, and session persistence; the desktop shell MUST provide the application window and desktop keyboard commands without independently duplicating those Harness responsibilities. FR-003–FR-009 and FR-011–FR-013 describe end-to-end capabilities supplied by dsh, not separate shell-owned implementations. Acceptance: complete User Stories 1–3 inside the shell and verify that their Harness configuration and session records remain owned by dsh; invoking the application quit command through its menu or Command+Q MUST follow FR-010.

- **FR-019**: During normal operation, closing any or all application windows MUST NOT quit the application, stop dsh, or cancel its tasks. Reopening a window MUST reconnect to the same running dsh instance without creating another instance or replaying work. A full application quit MUST stop the application's dsh instance; active work MUST receive the confirmation required by FR-010 even when no windows are open. Stopping an individual task under FR-009 MUST NOT stop dsh. Acceptance: User Story 1, scenarios 6–8, and the background-task and pending-approval edge cases.

- **FR-020**: The desktop shell MUST use Tauri and distribute Node.js and dsh with the installed application as sidecars. Launching Harness MUST NOT require a separately installed Node.js, dsh, or package manager, or download runtime dependencies on first launch. Acceptance: launch an installed build on a clean supported computer without those tools.
- **FR-021**: The desktop product's canonical version MUST equal the bundled dsh package version, including any prerelease suffix. Build and startup checks MUST reject a mismatched bundle before task execution. Acceptance: inspect the release manifest and About view, and attempt to build and launch a deliberately mismatched bundle.

- **FR-022**: The product MUST preferentially reference the official desktop implementation under the supplied `apps` source for in-scope desktop behavior and implementation decisions. Existing approved constraints MUST take precedence; reference availability alone MUST NOT authorize full feature parity or new capabilities. Planning MUST record the immutable reference revision, source locations, expected outcomes, adoption/adaptation/exclusion decisions, and reasons for differences across launch/readiness, window lifecycle, quit confirmation, menus/shortcuts, error recovery, and release compatibility. Technical reuse choices belong in the plan. Acceptance: User Story 4, scenarios 1 and 2.
- **FR-023**: Menu and keyboard routes for the same desktop action MUST produce equivalent outcomes. Repeated quit requests MUST share one unresolved confirmation; when Harness may be running but work status is unknown, quitting MUST warn about possible interruption. Acceptance: User Story 4, scenarios 3 and 4.
- **FR-024**: Adopted or adapted desktop behavior MUST have comparative acceptance evidence against the recorded official baseline, with intentional differences explained and unavailable or untested evidence labeled pending. Changes to the reference baseline or supported Harness release MUST trigger review of affected decisions and acceptance scenarios before adoption. Acceptance: User Story 4, scenarios 1 and 5.

### Key Entities *(include if feature involves data)*

- **Workspace**: A user-selected local project directory, its display name, availability, and associated sessions. Each session belongs to one workspace; changing its directory requires explicit confirmation.
- **Model Connection**: The supported model choice, credential, and configuration readiness used for tasks. Credentials are confidential and separate from conversation content.
- **Session**: A conversation associated with a workspace, including a user-recognizable title, creation and last-activity times, messages, task records, and history.
- **Task**: One submitted user request within a session, its execution state, reported actions, and final result or interruption reason.
- **Permission Decision**: The action and target requiring approval, its pending/allowed/denied state, and the user's explicit decision. An unresolved request grants no permission.
- **Application Preferences**: Shell-owned desktop preferences and safety-notice acknowledgement needed to restore the desktop experience. Model configuration, workspace associations, and conversation records remain dsh-owned data rather than duplicate shell records.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In a usability test with at least 10 target developers who have an installed application, an accessible sample project, and a valid credential, at least 90% can submit their first task within five minutes without terminal commands or facilitator assistance.
- **SC-002**: On the supported release-test computer with saved valid setup, at least 95% of 20 launches reach a usable workspace within 10 seconds. Model answer generation is not part of this launch measurement.
- **SC-003**: For at least 95% of 20 controlled task runs, new responses and reported action updates become visible within one second of becoming available to the application. Every Stop request in those runs receives a visible acknowledgement within one second.
- **SC-004**: In controlled tests, 100% of actions requiring approval remain unperformed before approval, and 100% of denied requests remain unperformed; all already-started actions are accurately reported after a Stop request.
- **SC-005**: Across 20 normal relaunches and 10 forced-exit recovery tests, all previously persisted session messages and workspace associations remain available, and no interrupted task or permission request resumes automatically. A saved session containing 1,000 messages opens for review within two seconds in at least 95% of 20 attempts.
- **SC-006**: In the usability test, at least 90% of participants independently complete a project-summary task, find its saved session after relaunch, and correctly identify whether a proposed action is awaiting approval or has completed; at least 80% rate task visibility and control at least 4 out of 5.

- **SC-007**: Across 10 controlled close/reopen cycles, including running tasks and pending approvals, 100% retain the same running dsh instance and current session without closure-induced cancellation, automatic permission decisions, or duplicated work. Across 10 confirmed full quits, the application's dsh instance is stopped when quitting completes; declining quit confirmation preserves both the application and dsh in every test.

- **SC-008**: Before affected implementation begins, all six desktop behavior areas in FR-022 have complete reference decisions, and 100% of adaptations or exclusions state their reason and user impact. At release review, all adopted or adapted behaviors have passing comparative evidence with no unexplained differences; pending evidence prevents an alignment-complete claim.
- **SC-009**: Across 10 quit trials covering repeated requests, unknown work status, and no open windows, every trial presents at most one unresolved confirmation, preserves work when declined, and stops the application's Harness operation before a confirmed quit completes.

## Assumptions

- The initial audience is individual developers working on local projects, not teams administering shared remote agents.
- "Refer wherever practical" means prefer the official desktop implementation while documenting justified adaptations, not replace the chosen shell or require identical appearance or full product parity. Existing Tauri, bundled Node.js/dsh, exact version equality, background lifetime, safety, and ownership requirements remain unchanged.
- Official reference: https://github.com/deepseek-ai/deepseek-harness/tree/master/apps. Initial inspection on 2026-10-06 identified revision `5badb15009ae1756c3afe0ae0cef1faafc290ccc`; `apps/desktop` and `apps/desktop-host` are relevant reference areas. Inspection alone does not establish runtime parity or compatibility with the bundled release.
- The existing plan and research must be reconciled with FR-022–FR-024 before affected implementation. Comparative verification depends on access to the official application or reproducible official behavior evidence; missing evidence remains pending.
- The initial supported operating system is macOS. This is a provisional scope assumption for the first release, not a requirement for future releases; Windows and Linux support are deferred unless the user changes this assumption before planning.
- Upstream already contains Web and Desktop applications, but this product provides its own desktop shell around the upstream dsh Web experience, not the upstream Desktop application. dsh is the upstream software kernel, not an operating-system kernel. A new agent engine, an independently rebuilt task interface, duplicate Harness data management, and feature parity with every upstream plugin are outside scope.
- The minimum useful release consists of desktop launch and local operation, one supported DeepSeek model connection, workspace selection, task progress and approvals, stopping, and persistent session history.
- Users supply valid model credentials, have network access for model-backed tasks, and grant access only to directories they choose. Model-service latency, quotas, billing, and answer quality are external dependencies, not desktop performance guarantees.
- A supported upstream release must be selected and verified during planning because Harness is in developer preview and warns of compatibility-breaking changes. Its existing task execution, persistence, and permission capabilities are dependencies to reuse and verify rather than recreate speculatively.
- Permission controls reduce risk but are not a security sandbox. Selecting a workspace provides task context, not a guarantee that every approved action is confined to that directory. Users remain responsible for backups and reviewing actions.
- Session history and preferences remain local until the user removes the application data; synchronized accounts, collaboration, remote-host connections, analytics, automatic updates, plugin discovery or installation, and custom plugin management are outside this release.
- Installation supplies a usable application; interactive task execution does not require the user to build Harness from source or manually manage its startup. Distribution mechanics and supported macOS versions are planning decisions.
- Closing a window and fully quitting the application are distinct operations. The shell manages its dsh instance for the application lifetime, not the window lifetime; unexpected service failures remain governed by the failure and recovery requirements.
- Test timings use an otherwise idle supported release-test computer with accessible local files. Planning must name that computer and test environment so measurements are reproducible.
- Source context reviewed on 2026-10-06: [upstream README](https://github.com/deepseek-ai/deepseek-harness/blob/master/README.md), [user guide](https://github.com/deepseek-ai/deepseek-harness/blob/master/docs/user/guide/index.md), [safety notice](https://github.com/deepseek-ai/deepseek-harness/blob/master/SAFETY.md), and [development guide](https://github.com/deepseek-ai/deepseek-harness/blob/master/docs/development.md). Repository artifacts follow the project constitution's English-only rule; the original user request was translated above.
