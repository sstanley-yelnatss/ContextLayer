# Example PR reasoning export

Exported from ContextLayer — demo fixture for trace CI when PR body has no reasoning block.

## PR Reasoning: auth token refresh

**Hypothesis:** Stale refresh token caused 401 loops after idle session.

**Action:** Reproduce with 24h idle fixture; trace token exchange in middleware.

**Evidence:** Logs show refresh returning 400 before retry storm; single retry fixes session.

**Conclusion:** confirmed — refresh endpoint needs grace window for concurrent tab refresh.
