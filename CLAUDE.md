# Fork workflow

This is a personal fork of zellij-org/zellij.

## Branch model

- `upstream/main`: source of truth for upstream development.
- `osc9-777`, `osc-1337`, `osc4-prefetch`, ...: feature branches for
  upstream PRs. Always rebased onto `upstream/main`. Each fork patch
  gets its OWN feature branch — never commit a patch straight to `main`
  (it's reset on every sync and the patch is lost).
- `local-tweaks`: branch holding fork-only files (this CLAUDE.md,
  `.claude/commands/`, etc.). Never PR'd upstream.
- `origin/main`: rebuilt artifact = `upstream/main` + `local-tweaks`
  + all feature branches merged in. Force-pushed.

## Don't

- Don't commit to `main` directly — it's reset on every sync.
- Don't add fork-only files to feature branches.
- Don't `git push --force` (use `--force-with-lease`).

## Sync workflow

Run `/sync-fork` to rebase feature branches onto upstream and rebuild
`origin/main`.
