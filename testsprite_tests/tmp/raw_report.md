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