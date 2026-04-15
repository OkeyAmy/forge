import requests
import json

BASE_URL = "http://localhost:5000"
HEADERS = {"Content-Type": "application/json"}
TIMEOUT = 30


def test_post_api_run_batch():
    # Valid batch request payload
    batch_payload = {
        "output_dir": "trajectories",
        "workers": 4,
        "items": [
            {"github_url": "https://github.com/owner/repo/issues/1"},
            {"repo": "owner/repo", "issue": 2},
            {"problem_text": "Fix off-by-one"}
        ]
    }

    # Test POST /api/run/batch success case
    response = requests.post(
        f"{BASE_URL}/api/run/batch",
        headers=HEADERS,
        data=json.dumps(batch_payload),
        timeout=TIMEOUT,
    )
    assert response.status_code == 200, f"Expected 200 but got {response.status_code}"
    resp_json = response.json()

    # Validate aggregated counts and keys
    assert resp_json.get("output_dir") == "trajectories"
    assert resp_json.get("total") == 3
    assert isinstance(resp_json.get("succeeded"), int)
    assert isinstance(resp_json.get("failed"), int)
    results = resp_json.get("results")
    assert isinstance(results, list)
    assert len(results) == 3

    # Validate individual results structure and values
    expected_instance_ids = {
        "https://github.com/owner/repo/issues/1",
        "https://github.com/owner/repo/issues/2",
        "text-Fix off-by-one",
    }
    found_instance_ids = set()
    success_count = 0
    failure_count = 0

    for result in results:
        instance_id = result.get("instance_id")
        success = result.get("success")
        error = result.get("error")

        assert instance_id in expected_instance_ids, f"Unexpected instance_id {instance_id}"
        found_instance_ids.add(instance_id)
        assert isinstance(success, bool)
        if success:
            assert error is None
            success_count += 1
        else:
            assert isinstance(error, str) and len(error) > 0
            failure_count += 1

    assert found_instance_ids == expected_instance_ids
    assert success_count == resp_json["succeeded"]
    assert failure_count == resp_json["failed"]
    assert resp_json["total"] == success_count + failure_count

    # If there are successful runs, verify at least one trajectory file exists via GET /api/trajectories
    if success_count > 0:
        trajectories_resp = requests.get(
            f"{BASE_URL}/api/trajectories",
            params={"dir": "trajectories"},
            timeout=TIMEOUT,
        )
        assert trajectories_resp.status_code == 200
        traj_json = trajectories_resp.json()
        assert traj_json.get("directory") == "trajectories"
        assert isinstance(traj_json.get("count"), int)
        assert isinstance(traj_json.get("trajectories"), list)
        assert traj_json["count"] >= success_count
        # Check that at least the trajectory files for succeeded runs are present in this list by name pattern
        names = {traj.get("name") for traj in traj_json["trajectories"]}
        assert any("owner__repo-i1" in name or "owner__repo-i2" in name for name in names)

    # Test malformed request body (missing items) returns 400
    malformed_payloads = [
        {},  # no items key
        {"output_dir": "trajectories"},  # items missing
        {"items": None},
        {"items": "not-an-array"},
    ]
    for malformed_body in malformed_payloads:
        resp = requests.post(
            f"{BASE_URL}/api/run/batch",
            headers=HEADERS,
            data=json.dumps(malformed_body),
            timeout=TIMEOUT,
        )
        assert resp.status_code == 400, f"Expected 400 for malformed body {malformed_body} but got {resp.status_code}"
        # Response should be plain string error message
        content_type = resp.headers.get("Content-Type", "")
        assert "application/json" in content_type or "text/plain" in content_type
        text = resp.text.strip()
        assert len(text) > 0, "Empty error message on 400 response"


test_post_api_run_batch()
