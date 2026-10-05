<!--
Sync Impact Report
==================
Version change: 1.0.0 → 1.1.0 (material expansion of development workflow guidance)

Modified principles: none

Modified sections:
- Development Workflow: require Conventional Commits and commit-message column limits.

Sections added: none
Sections removed: none

Deferred items / TODOs: none
-->

# dsh-dsh Constitution

## Core Principles

### I. English in Version Control (NON-NEGOTIABLE)

Everything that enters Git MUST be written in English. This includes source code,
identifiers, comments, documentation, specifications, commit messages, branch and tag
names, issue and pull request titles and bodies, and test fixtures. Files tracked in
the repository MUST NOT contain non-English prose as their primary content, with the
sole exception of assets where English cannot be represented (e.g. binary images,
locale data files consumed by third-party libraries).

Rationale: Git is the shared, long-lived system of record for dsh-dsh. A single
language keeps history searchable, reviewable, and greppable by every contributor and
by tooling, and it removes ambiguity in review comments and commit archaeology.
Localizing only at the presentation layer (i18n resource keys, message catalogs) is
the mechanism for supporting other languages; localizing the repository itself is not.

### II. Simplicity First

Before adding code, the implementer MUST walk the escalation ladder in `AGENTS.md` and
stop at the first rung that holds: does it need to exist at all (YAGNI), does the codebase
already provide it, does the standard library provide it, does the platform provide it, does
an installed dependency provide it, can it be a single line — and only then write the
minimum code that works. Abstraction, configuration, and indirection MUST be introduced
only against demonstrated need.

Rationale: dsh-dsh is a small desktop harness. Unnecessary structure raises the cost of
every future change and of every review, and cannot be recovered by refactoring later
without breaking interfaces.

## Language and Communication Scope

Principle I governs artifacts, not interaction. Natural-language communication with the
user — questions, explanations, summaries, and plans presented in the working session —
MAY be conducted in any language the user prefers, including languages other than English.

The boundary is the repository: text stays inside the repository only in English; text
delivered to the user in the session MAY follow the user's language. If the user requests
a session response in another language, the reply is given in that language while
repository artifacts remain English.

## Development Workflow

- Specification artifacts under `specs/` and `.specify/` are written in English, as are
  the plans and tasks derived from them.
- Behavior changes proceed specification-first: clarify intent, then plan, then implement.
  No change is merged that lacks a stated intent.
- Pull requests and reviews are conducted in English so that every reviewer, and the
  permanent record, can follow the discussion.
- Commit messages MUST follow [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/).
  The title (subject line) MUST NOT exceed 50 columns; each body line MUST NOT exceed
  72 columns. These limits keep commit history readable in terminal tools and reviews.
- Reviewers MUST verify commit-message format and column limits before merging.
- Every change MUST be accompanied by tests proportional to its blast radius.
- A reviewer MUST reject a contribution whose tracked files violate Principle I.

## Governance

This constitution supersedes other written practices in the repository. Where an existing
practice conflicts with it, this document wins until it is amended.

Amendments require:
1. A written proposal stating the principle added, changed, or removed and the rationale.
2. A version bump following semantic versioning, recorded in the Sync Impact Report that
   must accompany every amendment.
3. A migration plan for existing violations when the amendment is not backward compatible.

Versioning policy:
- MAJOR — removal of a principle, or redefinition of one in a backward incompatible way.
- MINOR — a new principle or section, or a material expansion of existing guidance.
- PATCH — clarifications, wording, and typo fixes with no semantic change.

Compliance is reviewed at every pull request and at every amendment. Any exception to
Principle I must be recorded in the pull request description with the reason, and MUST
satisfy the sole exception stated in that principle.

**Version**: 1.1.0 | **Ratified**: 2026-10-05 | **Last Amended**: 2026-10-05
