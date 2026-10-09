# User-led manual candidate handoff

Source checkpoint `2bdf282`. Current release desktop binary built and copied into
the release bundle, then finalized using the existing acceptance-only signing/hash
procedure. Installed copy passed deep/strict codesign and native closure
verification. Exact binary/manifest hashes and paths: `manual-candidate-data.json`.
This is a release build with no debug fixture consent/entry/Inspector/carrier
controls required, not the older production candidate or the debug carrier page.
At handoff, no user manual case had run and no provider/model/task request had
been submitted by the maintainer. Subsequent user-reported M01–M03 passes are
recorded separately in `manual-results.md`; preparation hashes are not runtime
measurement evidence.

Candidate: `/private/tmp/dsh manual install.bZA0oI/DeepSeek Harness Desktop.app`.
Isolated disposable HOME: `/private/tmp/dsh manual home.fq2SK5`.
Disposable workspaces: `/private/tmp/dsh manual workspaces.GzZlzZ/A` and `B`.
Directories are owner-only; seeded README files contain no project data or secrets.
Do not use Finder to launch this isolated test: it would not carry the supplied
HOME and could use the normal account's application data. Keep this terminal
open and use the same profile command for relaunches:

```sh
/usr/bin/env -i HOME='/private/tmp/dsh manual home.fq2SK5' USER=john LOGNAME=john PATH=/usr/bin:/bin TMPDIR=/private/tmp '/private/tmp/dsh manual install.bZA0oI/DeepSeek Harness Desktop.app/Contents/MacOS/deepseek-harness-desktop'
```

The line is also saved locally at `/private/tmp/dsh-manual-start.txt`. No separate
Node/npm/dsh is invoked. Clearing inherited environment avoids accidental debug
fixture flags and ambient provider credentials; model configuration stays in
upstream UI. This command is specific to this prepared host/profile, not a
portable installer or production account migration procedure.

Follow [manual.md](../manual.md), M01–M03 first. Report Pass/Fail/Blocked/Not run
with concrete observations and no credentials. Preparation and source/tooling
checks do not constitute an installed startup/task pass. SC-002/SC-007 and task
checkmarks remain unchanged until their actual required evidence exists.

Local ad-hoc signing is not Developer ID/notarization or public Gatekeeper
qualification. Temporary candidate paths may be cleaned by the OS; this is
acceptance-only delivery, not T052 packaging automation/DMG. No system security
controls were disabled. User-led testing replaces interactive probe expansion
for this round, not remaining automated native security/byte/identity checks.
