# Open defect — reversed composer input after response completion

Status: **intermittent user-reported defect, not independently reproduced or fixed**.
The user subsequently reported normal input again, without a known cause. That
is not verification of a fix.
User-led release candidate handoff: source `2bdf282`, see `manual-candidate.md`.
Candidate/OS/input source were not reconfirmed at this report.

## Report and observation

After the answer finishes, input behaves as if typing right-to-left. Focusing
another surface and refocusing the composer restores normal behavior. The supplied
screenshot shows a composer draft `ti evomeR`, consistent with reversed characters
if the intended phrase was `Remove it`; intended keystrokes are not yet confirmed.
The screenshot alone cannot distinguish bidi layout, a repeatedly reset insertion
caret, IME composition or editor-selection synchronization. The private screenshot
is not copied into the repository; surrounding model narrative is not a tool trace.

Need to confirm: direct Latin input source versus Chinese IME, Enter submission
versus send-button submission, frequency and whether input typed while streaming
also exhibits the symptom. No extra provider call is requested solely to answer
these questions if the user already knows the behavior.

## Subsequent terminal observation

User supplied a macOS message: `error messaging the mach port for
IMKCFRunLoopWakeUpReliable`, followed by a terminal Ctrl-C. The name refers to
InputMethodKit/input-method run-loop messaging. This isolated message does not
establish the cause of the reversed caret/input symptom, nor prove a crash or
harmlessness. Whether recovery occurred in the same run or after restarting was
not specified. Restarting could reset transient editor/input-method state, but
that remains a hypothesis.

For normal testing, use native Quit/Stop and Quit instead of Ctrl-C; forced
terminal interruption is not the confirmed-quit acceptance path. Do not mark
this defect fixed solely because it stopped reproducing. Other manual cases
may continue with this open intermittent defect retained. If it recurs, record
submission route, input source, focus/hide-show history and whether recovery
requires restarting; no credentials or raw editor contents are needed.

## Static inspection, not causal proof

- Own `src/harness.ts`, `transport.ts` and `native-transport.ts` do not rewrite the
  composer, input direction, DOM selection or caret on response completion.
- Native `window/native.rs` calls set_focus for create/show, not per model update.
  That does not rule out broader native/WK focus behavior.
- Pinned R `dsh-client-ui-conversation/lib/client.js` owns the composer using
  Lexical, a resident `[data-composer-input]` contenteditable and editor binding.
  `ComposerContentEditable` around lines 16540–16570 sets its root and editability.
- `focusDraftEditor` around lines 16763–16774 performs DOM focus plus editor.focus.
  The draft shell's focus method around lines 13816–13823 explicitly notes that
  bare DOM focus can put the caret at the start instead of restoring selection.
  This makes caret/selection synchronization a plausible lead, **not a diagnosis**.

Do not patch downloaded R assets, force global LTR, move the caret to the end on
every keystroke, or blur/refocus on interpreted task state without a failing
reproduction. Those could break selections, intentional bidi text, IME, legitimate
mid-draft editing and upstream ownership. A narrow shell/WK compatibility fix
requires a reproduction and regression; an upstream defect stays attributed only
after comparative evidence, not source inspection alone.

## Next verification

Use a controlled disposable response and harmless unsent text to observe native
input and DOM caret offsets before/after completion, with focus/blur as a control.
If diagnostic capture is needed, record only lengths, offsets, inputType,
composition/focus flags and computed direction—not draft text, keys, settings,
reply bodies or credentials. Compare a plain standalone pinned-R browser flow
where feasible. Confirm direct Latin typing, Chinese composition, mid-string
selection/replacement, Enter/button submission and retained hide/show after any fix.
No desktop automation runs while the user is manually interacting.

M04–M06 remain user-reported workflow passes; this defect is tracked independently
and blocks claiming the tested composer interaction is fully qualified. T032/T033
and remaining manual cases are not advanced by this report.
