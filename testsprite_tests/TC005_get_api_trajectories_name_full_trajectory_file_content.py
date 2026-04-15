import requests
import time

BASE_URL = "http://localhost:5000"
TIMEOUT = 30

def test_get_api_trajectories_name_full_trajectory_file_content():
    # First, create a new trajectory file by POST /api/run to have a valid trajectory filename
    run_payload = {
        "github_url": "https://github.com/owner/repo/issues/42",
        "output_dir": "trajectories"
    }

    headers = {
        "Content-Type": "application/json"
    }

    trajectory_name = None
    try:
        run_resp = requests.post(f"{BASE_URL}/api/run", json=run_payload, headers=headers, timeout=TIMEOUT)
        assert run_resp.status_code == 200, f"POST /api/run failed with status {run_resp.status_code}"
        run_data = run_resp.json()
        assert "trajectory_file" in run_data, "Response missing trajectory_file"
        trajectory_name = run_data["trajectory_file"].split("/", 1)[-1]  # e.g., "owner__repo-i42.traj"
        assert trajectory_name.endswith(".traj"), "trajectory_file does not end with .traj"

        # Test GET /api/trajectories/{name} with valid trajectory filename and directory
        params = {"dir": "trajectories"}
        get_resp = requests.get(f"{BASE_URL}/api/trajectories/{trajectory_name}", params=params, timeout=TIMEOUT)
        assert get_resp.status_code == 200, f"GET /api/trajectories/{trajectory_name} returned {get_resp.status_code}"
        traj_data = get_resp.json()

        # Validate expected keys in full TrajFile JSON response
        assert isinstance(traj_data, dict), "Trajectory response is not a JSON object"
        # trajectory steps should be present
        assert "steps" in traj_data and isinstance(traj_data["steps"], list), "Missing or invalid steps"
        # agent info should be present
        assert "agent" in traj_data and isinstance(traj_data["agent"], dict), "Missing or invalid agent info"
        # environment info should be present
        assert "env" in traj_data and isinstance(traj_data["env"], dict), "Missing or invalid environment info"
        # submission patch should be present
        assert "submission_patch" in traj_data, "Missing submission_patch"

        # Test 404 Not Found for missing file
        missing_filename = "nonexistent-file.traj"
        resp_404 = requests.get(f"{BASE_URL}/api/trajectories/{missing_filename}", params=params, timeout=TIMEOUT)
        assert resp_404.status_code == 404, f"Expected 404 for missing file, got {resp_404.status_code}"

        # To test 422 parse error, try to request with a filename that might cause parse error
        # Forge-api does not specify what filename triggers 422, so we simulate by sending an invalid filename format.
        parse_error_filename = "corrupt-file.traj"
        resp_422 = requests.get(f"{BASE_URL}/api/trajectories/{parse_error_filename}", params=params, timeout=TIMEOUT)
        # Accept either 422 or 404 depending on implementation, but per spec 422 is expected on parse error
        if resp_422.status_code != 422:
            # It may return 404 if file absent, so only assert if it is 422 or 404
            assert resp_422.status_code in (404, 422), f"Expected 422 or 404 for parse error test file, got {resp_422.status_code}"

    finally:
        # Cleanup: delete the created trajectory file if possible by deleting the directory or file
        # The API does not document DELETE for trajectories, so no deletion API exists.
        # So no cleanup steps possible via API.
        pass


test_get_api_trajectories_name_full_trajectory_file_content()