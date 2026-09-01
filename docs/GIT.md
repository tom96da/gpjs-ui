<!--
Copyright (c) 2026 tom96da
SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Git workflow

Conventions and hard rules for commits and other git operations in this
repository. Anyone working here — AI agents included — follows these.

## Branching model

This repo's target branching model is the standard [Git-Flow](https://nvie.com/posts/a-successful-git-branching-model/), described below.

**During initial development, before there are real releases to manage, this repo uses [GitHub Flow](https://docs.github.com/en/get-started/using-github/github-flow) instead**: just `main`, with short-lived branches merged directly into it via review. There's no `develop`/`release`/`hotfix` branch yet. Switch to the full Git-Flow model once the project starts cutting actual releases.

### Git-Flow (target, once releases start)

Long-lived branches:

- `main` — always production-ready. Every commit on `main` is a release and gets tagged `vX.Y.Z`.
- `develop` — integration branch for completed work heading into the next release.

Supporting branches, all short-lived and deleted after merging:

- `feature/<name>` — branched from `develop`, merged back into `develop`. Never merged directly into `main`.
- `release/<version>` — branched from `develop` when preparing a release (version bump, docs, final fixes only — no new features). Merged into both `main` (tagged `vX.Y.Z`) and back into `develop`.
- `hotfix/<name>` — branched from `main` for an urgent production fix. Merged into both `main` (tagged) and `develop`.

Never commit directly to `main` or `develop` — land work through a supporting branch, merged in via review.

## Never commit without review

Committing (`git commit`, `git commit --amend`, or anything else that
creates/rewrites history) is never done unilaterally. Before running any
commit, show the exact staged diff and the exact final commit message, and
get explicit approval of that specific content — agreement that "committing
is the next step" in general is not the same as approval of the actual
diff/message. Preparing a commit and reporting it afterward is backwards;
review happens before the commit exists, not after.

## Amending vs. new commits

Prefer a new commit over amending. Amending is acceptable only when
explicitly requested, and only for a commit that hasn't been pushed anywhere
shared — the review rule above applies to amends exactly the same as to new
commits.

## Commit message format

Follows [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

- `type` is one of `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`.
- Before picking a `type`/`scope`, check existing precedent with `git log --oneline -- <path>` for the area being touched, and match it — e.g. this repo's devcontainer changes are `chore(devcontainer)`, not `build(devcontainer)`.
- `description` is imperative, lower-case, no trailing period (e.g. `feat(runtime): add quickjs bridge`).
- `body` is a concise bullet list of what was done and why — not prose. Each bullet follows the same style as the description: starts lower-case unless the first word is a proper noun (a filename, package name, etc.), and has no trailing period.
- A breaking change is marked either with `!` after the type/scope (`feat!: ...`) or a `BREAKING CHANGE:` footer — not both unless it aids clarity.
- Scope is optional; use it for the affected area once the workspace has named crates/packages (e.g. `fix(gpui-shell): ...`).
- Any commit Claude is involved in must include a `Co-Authored-By: Claude <noreply@anthropic.com>` trailer (adjust the model name if relevant, e.g. `Claude Sonnet 5`).
