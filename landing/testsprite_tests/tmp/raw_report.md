# TestSprite AI Testing Report(MCP)

---

## 1️⃣ Document Metadata
- **Project Name:** forge-landing
- **Date:** 2026-04-16
- **Prepared by:** TestSprite AI Team

---

## 2️⃣ Requirement Validation Summary

#### Test TC001 Hero content renders on first load
- **Test Code:** [TC001_Hero_content_renders_on_first_load.py](./TC001_Hero_content_renders_on_first_load.py)
- **Test Visualization and Result:** https://www.testsprite.com/dashboard/mcp/tests/ae11ebe3-8a04-409b-8fff-9c8d8b46f96a/aeaab406-1730-451e-af38-a08857a8abb5
- **Status:** ✅ Passed
- **Analysis / Findings:** Hero headline and supporting copy render correctly on initial load.
---

#### Test TC002 Setup section reachable by in-page navigation or scroll
- **Test Code:** [TC002_Setup_section_reachable_by_in_page_navigation_or_scroll.py](./TC002_Setup_section_reachable_by_in_page_navigation_or_scroll.py)
- **Test Visualization and Result:** https://www.testsprite.com/dashboard/mcp/tests/ae11ebe3-8a04-409b-8fff-9c8d8b46f96a/7bc65d08-3663-42f1-9252-b546b802efbb
- **Status:** ✅ Passed
- **Analysis / Findings:** The Setup section is reachable via hash/scroll and the default Docker instructions are visible.
---

#### Test TC003 Model configuration section is visible with example snippet
- **Test Code:** [TC003_Model_configuration_section_is_visible_with_example_snippet.py](./TC003_Model_configuration_section_is_visible_with_example_snippet.py)
- **Test Visualization and Result:** https://www.testsprite.com/dashboard/mcp/tests/ae11ebe3-8a04-409b-8fff-9c8d8b46f96a/a31afcaa-5ee2-452a-bbf4-9cee66a68048
- **Status:** ✅ Passed
- **Analysis / Findings:** Provider labels and a non-empty configuration example are visible without clicking tabs.
---

#### Test TC004 Setup mode controls are visible (no toggle verification)
- **Test Code:** [TC004_Setup_mode_controls_are_visible_no_toggle_verification.py](./TC004_Setup_mode_controls_are_visible_no_toggle_verification.py)
- **Test Visualization and Result:** https://www.testsprite.com/dashboard/mcp/tests/ae11ebe3-8a04-409b-8fff-9c8d8b46f96a/d649cc5e-8d6a-47aa-85e4-c04b180742b2
- **Status:** ✅ Passed
- **Analysis / Findings:** Docker and Build from source controls are present and visible as expected.
---

#### Test TC005 GitHub repository links use new tab and safe rel (DOM or single click)
- **Test Code:** [TC005_GitHub_repository_links_use_new_tab_and_safe_rel_DOM_or_single_click.py](./TC005_GitHub_repository_links_use_new_tab_and_safe_rel_DOM_or_single_click.py)
- **Test Error:** TEST BLOCKED

The feature could not be reached — the runner could not inspect the GitHub anchor attributes on the landing page.

Observations:
- A GitHub anchor was clicked, but the DOM attribute values (`href`, `target`, `rel`) were not accessible.
- The runner did not expose `data-testid` or attribute values for the GitHub anchors, and the external GitHub page did not load with attribute details available.
- **Test Visualization and Result:** https://www.testsprite.com/dashboard/mcp/tests/ae11ebe3-8a04-409b-8fff-9c8d8b46f96a/cdca876a-8d6d-4662-bf74-0beaa03c800e
- **Status:** BLOCKED
- **Analysis / Findings:** This is a runner limitation, not evidence of broken landing markup.
---

#### Test TC006 How it works section visible after scroll or hash
- **Test Code:** [TC006_How_it_works_section_visible_after_scroll_or_hash.py](./TC006_How_it_works_section_visible_after_scroll_or_hash.py)
- **Test Visualization and Result:** https://www.testsprite.com/dashboard/mcp/tests/ae11ebe3-8a04-409b-8fff-9c8d8b46f96a/5fb979f7-c986-4054-bde1-ab0bf7dcd56f
- **Status:** ✅ Passed
- **Analysis / Findings:** The section heading, steps, and terminal region are visible after scroll/hash navigation.
---

#### Test TC007 Linear journey: features through footer without interactive toggles
- **Test Code:** [TC007_Linear_journey_features_through_footer_without_interactive_toggles.py](./TC007_Linear_journey_features_through_footer_without_interactive_toggles.py)
- **Test Visualization and Result:** https://www.testsprite.com/dashboard/mcp/tests/ae11ebe3-8a04-409b-8fff-9c8d8b46f96a/73b8ef6b-05f9-47cc-b7ba-84edb541b660
- **Status:** ✅ Passed
- **Analysis / Findings:** Each major landing section is visible when navigated in sequence.
---

#### Test TC008 Footer branding and external link markup
- **Test Code:** [TC008_Footer_branding_and_external_link_markup.py](./TC008_Footer_branding_and_external_link_markup.py)
- **Test Visualization and Result:** https://www.testsprite.com/dashboard/mcp/tests/ae11ebe3-8a04-409b-8fff-9c8d8b46f96a/c926263c-0353-4e81-a384-ea62c0118909
- **Status:** ✅ Passed
- **Analysis / Findings:** Footer branding and external link markup checks passed in the latest run.
---

## 3️⃣ Coverage & Matching Metrics

- **87.50** of tests passed

| Requirement | Total Tests | ✅ Passed | 🚫 Blocked | ❌ Failed |
|-------------|-------------|-----------|------------|----------|
| Landing page frontend | 8 | 7 | 1 | 0 |

---

## 4️⃣ Key Gaps / Risks
- **TC005 blocked:** GitHub anchor verification is limited by the cloud runner’s DOM/external-page access, not by evidence of a broken landing page.
- **Runner instability:** Prior tunnel/localhost timeouts can still affect re-runs; production serving remains the safest mode.
- **Legacy generated scripts:** Old click-heavy files still exist beside the current 8-case plan and can cause confusion if read as current truth.

---
<contents copied from landing raw_report>

# TestSprite AI Testing Report(MCP)

---

## 1️⃣ Document Metadata
- **Project Name:** SWEagent
- **Date:** 2026-04-15
- **Prepared by:** TestSprite AI Team

---

## 2️⃣ Requirement Validation Summary

#### Test TC001 post api run single forge agent run
- **Test Code:** [TC001_post_api_run_single_forge_agent_run.py](./TC001_post_api_run_single_forge_agent_run.py)
- **Test Error:** Traceback (most recent call last):
  File "/var/task/handler.py", line 258, in run_with_retry
    exec(code, exec_env)
  File "<string>", line 61, in <module>
  File "<string>", line 18, in test_post_api_run_single_forge_agent_run
AssertionError: Expected 200 OK but got 500

- **Test Visualization and Result:** https://www.testsprite.com/dashboard/mcp/tests/fe4fc7e2-5706-4c12-b913-245a7ebfb283/a8923ef8-9e9c-4995-98db-167d9d8f084c
- **Status:** ❌ Failed
- **Analysis / Findings:** {{TODO:AI_ANALYSIS}}.
---

#### Test TC002 post api run batch parallel agent runs
- **Test Code:** [TC002_post_api_run_batch_parallel_agent_runs.py](./TC002_post_api_run_batch_parallel_agent_runs.py)
- **Test Visualization and Result:** https://www.testsprite.com/dashboard/mcp/tests/fe4fc7e2-5706-4c12-b913-245a7ebfb283/7d6e049e-f235-428d-9e40-6c3d9bb5f0a6
- **Status:** ✅ Passed
- **Analysis / Findings:** {{TODO:AI_ANALYSIS}}.
---

#### Test TC003 post get delete api watch background github label watcher
- **Test Code:** [TC003_post_get_delete_api_watch_background_github_label_watcher.py](./TC003_post_get_delete_api_watch_background_github_label_watcher.py)
- **Test Visualization and Result:** https://www.testsprite.com/dashboard/mcp/tests/fe4fc7e2-5706-4c12-b913-245a7ebfb283/f1cc0d95-3900-4d74-a151-04ee713fffa9
- **Status:** ✅ Passed
- **Analysis / Findings:** {{TODO:AI_ANALYSIS}}.
---

#### Test TC004 get api trajectories list trajectory files with metadata
- **Test Code:** [TC004_get_api_trajectories_list_trajectory_files_with_metadata.py](./TC004_get_api_trajectories_list_trajectory_files_with_metadata.py)
- **Test Visualization and Result:** https://www.testsprite.com/dashboard/mcp/tests/fe4fc7e2-5706-4c12-b913-245a7ebfb283/32d9e381-6633-4113-8cb9-7821954343b2
- **Status:** ✅ Passed
- **Analysis / Findings:** {{TODO:AI_ANALYSIS}}.
---

#### Test TC005 get api trajectories name full trajectory file content
- **Test Code:** [TC005_get_api_trajectories_name_full_trajectory_file_content.py](./TC005_get_api_trajectories_name_full_trajectory_file_content.py)
- **Test Error:** Traceback (most recent call last):
  File "/var/task/handler.py", line 258, in run_with_retry
    exec(code, exec_env)
  File "<string>", line 65, in <module>
  File "<string>", line 21, in test_get_api_trajectories_name_full_trajectory_file_content
AssertionError: POST /api/run failed with status 500

- **Test Visualization and Result:** https://www.testsprite.com/dashboard/mcp/tests/fe4fc7e2-5706-4c12-b913-245a7ebfb283/75b6712d-8d71-49b6-b734-16a194817e83
- **Status:** ❌ Failed
- **Analysis / Findings:** {{TODO:AI_ANALYSIS}}.
---

#### Test TC006 get api stats aggregate exit status counts
- **Test Code:** [TC006_get_api_stats_aggregate_exit_status_counts.py](./TC006_get_api_stats_aggregate_exit_status_counts.py)
- **Test Visualization and Result:** https://www.testsprite.com/dashboard/mcp/tests/fe4fc7e2-5706-4c12-b913-245a7ebfb283/adfd7a85-cb0b-49c5-8780-cfda458e80b8
- **Status:** ✅ Passed
- **Analysis / Findings:** {{TODO:AI_ANALYSIS}}.
---

#### Test TC007 get api issues list open github issues
- **Test Code:** [TC007_get_api_issues_list_open_github_issues.py](./TC007_get_api_issues_list_open_github_issues.py)
- **Test Visualization and Result:** https://www.testsprite.com/dashboard/mcp/tests/fe4fc7e2-5706-4c12-b913-245a7ebfb283/6370acfe-c632-446a-ada5-f660c70dfc83
- **Status:** ✅ Passed
- **Analysis / Findings:** {{TODO:AI_ANALYSIS}}.
---

#### Test TC008 get health liveness readiness probe
- **Test Code:** [TC008_get_health_liveness_readiness_probe.py](./TC008_get_health_liveness_readiness_probe.py)
- **Test Visualization and Result:** https://www.testsprite.com/dashboard/mcp/tests/fe4fc7e2-5706-4c12-b913-245a7ebfb283/0c305c89-3a9b-45c8-b98c-72f4ab891373
- **Status:** ✅ Passed
- **Analysis / Findings:** {{TODO:AI_ANALYSIS}}.
---


## 3️⃣ Coverage & Matching Metrics

- **75.00** of tests passed

| Requirement        | Total Tests | ✅ Passed | ❌ Failed  |
|--------------------|-------------|-----------|------------|
| ...                | ...         | ...       | ...        |
---


## 4️⃣ Key Gaps / Risks
{AI_GNERATED_KET_GAPS_AND_RISKS}
---