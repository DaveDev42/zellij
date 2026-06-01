---
description: Rebase feature branches on upstream/main and rebuild origin/main
model: sonnet
---

Sync this fork. Read CLAUDE.md first if you haven't — it explains the branch model.

Branches:
- local-only: `local-tweaks`
- feature: `osc9-777`, `osc-1337`, `osc4-prefetch`, `active-pane-dim`, `window-dim`

## Steps

1. `git fetch upstream`

2. Rebase each branch onto `upstream/main`. For each of `local-tweaks`, `osc9-777`, `osc-1337`, `osc4-prefetch`, `active-pane-dim`, `window-dim` (rebase `window-dim` last — it sits on top of `active-pane-dim`):
   - `git checkout <branch>`
   - `git rebase upstream/main`
   - If conflict: stop and report. Do not resolve on your own.
   - `git push origin <branch> --force-with-lease`
   - For feature branches, the push updates the open PR — that's expected.

3. Rebuild `origin/main`:
   - `git checkout main`
   - Sanity check: `git log upstream/main..main --oneline`. If there are commits here that aren't in any feature/local branch, STOP and report — those would be lost. (Reset is destructive, but only safe to run once we know nothing direct-committed to main.)
   - `git reset --hard upstream/main`
   - `git merge --no-ff local-tweaks`
   - `git merge --no-ff osc9-777`
   - `git merge --no-ff osc-1337`
   - `git merge --no-ff osc4-prefetch`
   - `git merge --no-ff active-pane-dim`
   - `git merge --no-ff window-dim`
   - If a merge conflict appears: rerere may have auto-resolved it (check `git status`). If unresolved, stop and report.
   - `git push origin main --force-with-lease`

4. Final summary: `git log --oneline -10` and `git status`. Report which branches were rebased, whether any conflicts needed manual help, and final main HEAD.

## Rules

- Use `--force-with-lease` only. Never plain `--force`.
- Do not add or remove branches from the list. The user edits this file when that changes.
- Do not touch `upstream/*` refs (they're read-only).
- If anything looks off (unexpected commits on main, dirty working tree at start, push rejected), stop and ask before continuing.
