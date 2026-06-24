# GitHub settings (one-time, in the browser)

Do these after `develop` exists on the remote. Takes ~5 minutes.

## Default branch

**Settings → General → Default branch**

Keep **`main`** as default so the repo homepage shows the stable line.

## Branch protection — `main`

**Settings → Branches → Add branch ruleset** (or classic rule)

Target: `main`

Recommended:

- Require a pull request before merging
- Allow bypass only for admins (optional; leave off if you want the rule to apply to you too)
- Require status checks: **`test`** (from CI workflow)
- Do not allow force pushes

Day-to-day work merges to **`develop`**. Release by opening **PR: `develop` → `main`**, then tag on `main` (e.g. `v0.1.0`).

## Branch protection — `develop` (optional)

Lighter rule: require PR or allow direct push (solo dev). CI still runs on push.

## Releases

When `main` gets a milestone:

1. Merge PR `develop` → `main`
2. **Releases → Draft new release** → tag `v0.x.0` from `main`
3. Paste a short changelog (PR export, MCP tools, etc.)
