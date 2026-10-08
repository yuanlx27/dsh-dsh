---
name: DeepSeek Harness Desktop Shell
colors:
  background: "#f5f5f7"
  surface: "#ffffff"
  text: "#1d1d1f"
  primary: "#175cd3"
  on-primary: "#ffffff"
  border: "#d2d2d7"
  error: "#b42318"
typography:
  body:
    fontFamily: system-ui
    fontSize: 16px
    fontWeight: "400"
    lineHeight: 24px
rounded:
  sm: 6px
  md: 12px
spacing:
  unit: 4px
  md: 16px
  lg: 24px
---

# Shell design guidance

## Evidence and scope

The specification asks for a small semantic macOS startup, safety and failure
surface, not a redesigned Harness. No brand/mockup tokens were provided. The
values above are conservative recommendations for this shell only; preserve
upstream Web UI styling, keyboard behavior and accessibility unmodified.

## Theme, palette and typography

Use calm opaque system surfaces, dark readable text, a blue labelled primary
action and red labelled errors. No gradients, illustrations, remote fonts,
analytics or decorative animations. Use system-ui, 16/24px body text, 24px bold
headings and readable line lengths. Status and errors must have text, not color
alone. Do not show user configuration, secrets or raw process diagnostics.

## Layout, shapes and controls

A centred single column, maximum 40rem, 24px page padding, 16px gaps. At narrow
sizes let actions wrap; keep text and dialogs scrollable at zoom. Use 6px button
and 12px panel corners, a thin border and no required shadow. Buttons have at
least a 44px height. The shell has no conversation, account, workspace or
permission-editor components; entry leads to upstream Harness.

## State, focus and motion

Native semantic buttons and a modal `<dialog>` provide keyboard behavior.
Use a visible 3px blue focus outline with 3px offset. Required notice has explicit
acknowledgement and a non-accepting dismiss action; dismissal never grants entry.
Focus the dialog when presented, return focus to its initiating control and use
polite live status text. Do not auto-focus on polling updates. Failed operations
keep entry disabled and offer a fixed next action. Motion is unnecessary;
respect reduced-motion if future transitions are added. Actual VoiceOver/native
focus behavior still requires installed macOS qualification.
