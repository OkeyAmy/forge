import requests


def test_post_api_run_single_forge_agent_run():
    base_url = "http://localhost:5000"
    headers = {"Content-Type": "application/json"}
    timeout = 30

    # Valid request body with github_url and output_dir
    valid_payload = {
        "github_url": "https://github.com/owner/repo/issues/42",
        "output_dir": "trajectories"
    }

    # 1. Test successful POST /api/run with valid payload
    try:
        response = requests.post(f"{base_url}/api/run", json=valid_payload, headers=headers, timeout=timeout)
        assert response.status_code == 200, f"Expected 200 OK but got {response.status_code}"
        data = response.json()

        # Validate response fields
        assert "exit_status" in data, "Missing exit_status in response"
        assert data["exit_status"] == "submitted", f"exit_status expected to be 'submitted', got {data['exit_status']}"
        assert "has_submission" in data and data["has_submission"] is True, "has_submission should be True"
        assert "submission_preview" in data and isinstance(data["submission_preview"], str) and data["submission_preview"], "submission_preview should be a non-empty string"
        assert "steps" in data and isinstance(data["steps"], int) and data["steps"] > 0, "steps should be a positive integer"
        assert "model_stats" in data and isinstance(data["model_stats"], dict), "model_stats missing or not a dict"
        assert "total_cost" in data["model_stats"] and isinstance(data["model_stats"]["total_cost"], (int, float)), "model_stats.total_cost missing or not a number"
        assert "trajectory_file" in data and isinstance(data["trajectory_file"], str) and data["trajectory_file"], "trajectory_file should be a non-empty string and path"

    except requests.RequestException as e:
        assert False, f"RequestException during valid POST /api/run: {e}"

    # 2. Test error case POST /api/run with missing problem source fields
    # Missing github_url, repo+issue, and problem_text fields
    invalid_payload = {
        "output_dir": "trajectories"
    }

    try:
        error_response = requests.post(f"{base_url}/api/run", json=invalid_payload, headers=headers, timeout=timeout)
        assert error_response.status_code == 400, f"Expected 400 Bad Request for missing problem source fields but got {error_response.status_code}"
        # Expect response to be an error message string
        # Can be plain text or JSON string; attempt to parse JSON else fallback to text
        try:
            error_data = error_response.json()
            # If parsed JSON, should be a string or have some error message
            if isinstance(error_data, dict):
                # some APIs return { "error": "..."} - accept that but PRD says string
                assert any(isinstance(v, str) for v in error_data.values()), "Expected error message string in JSON response"
            else:
                assert isinstance(error_data, str), "Expected error message string in json response"
        except ValueError:
            # Not JSON, assume plain text
            assert isinstance(error_response.text, str) and error_response.text.strip() != "", "Expected error message string in text response"

    except requests.RequestException as e:
        assert False, f"RequestException during invalid POST /api/run: {e}"


test_post_api_run_single_forge_agent_run()