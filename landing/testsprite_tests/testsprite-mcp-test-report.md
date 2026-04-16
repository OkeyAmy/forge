## TestSprite MCP — Frontend test report (forge-landing)

1) Document metadata

- Project: forge-landing (Next.js marketing site in `landing/`)
- Date: 2026-04-16
- Scope: Landing page only
- Latest project id: `ae11ebe3-8a04-409b-8fff-9c8d8b46f96a`

---

2) Results summary

- Total executed: 8
- Passed: 7
- Blocked: 1
- Failed: 0
- Pass rate: 87.5%

Per-test status:
- TC001 — PASS
- TC002 — PASS
- TC003 — PASS
- TC004 — PASS
- TC005 — BLOCKED
- TC006 — PASS
- TC007 — PASS
- TC008 — PASS

Blocked detail:
- TC005 could not inspect GitHub anchor attributes in the cloud runner. The runner clicked a GitHub link but did not expose DOM attribute values and did not provide a usable external page state.

---

3) Coverage & metrics

- Current canonical landing plan: 8 tests from `landing/testsprite_tests/testsprite_frontend_test_plan.json`
- Current outcome: 7/8 passed, 1/8 blocked, 0 failed
- Outdated backend/root report data should not be used for landing evaluation

---

4) Key gaps / actions

- Use the landing artifacts as source of truth:
  - `landing/testsprite_tests/tmp/test_results.json`
  - `landing/testsprite_tests/tmp/raw_report.md`
  - `landing/testsprite_tests/testsprite-mcp-test-report.md`
- Ignore older backend-style results in root `testsprite_tests/` when discussing the landing page
- If needed, re-run only TC005 with a DOM-inspection-friendly runner configuration

---
