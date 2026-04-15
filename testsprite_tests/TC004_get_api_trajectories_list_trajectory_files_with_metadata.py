import requests

BASE_URL = "http://localhost:5000"
TIMEOUT = 30

def test_get_api_trajectories_list_with_metadata():
    # Test valid directory query parameter
    valid_dir = "trajectories"
    url = f"{BASE_URL}/api/trajectories"
    params = {"dir": valid_dir}
    try:
        response = requests.get(url, params=params, timeout=TIMEOUT)
        assert response.status_code == 200, f"Expected 200 OK for existing directory, got {response.status_code}"
        data = response.json()
        # Validate directory field
        assert data.get("directory") == valid_dir, f"Expected directory '{valid_dir}', got {data.get('directory')}"
        # Validate count is int and non-negative
        count = data.get("count")
        assert isinstance(count, int) and count >= 0, f"Expected count to be non-negative int, got {count}"
        # Validate trajectories is a list
        trajectories = data.get("trajectories")
        assert isinstance(trajectories, list), f"Expected trajectories to be a list, got {type(trajectories)}"
        for traj in trajectories:
            # Validate trajectory fields and types
            assert "name" in traj and isinstance(traj["name"], str), "Trajectory is missing 'name' or it is not a string"
            # exit_status may be empty string but must be string type
            assert "exit_status" in traj and isinstance(traj["exit_status"], str), "Trajectory missing 'exit_status' or not a string"
            assert "has_submission" in traj and isinstance(traj["has_submission"], bool), "Trajectory missing 'has_submission' or not a bool"
            assert "steps" in traj and isinstance(traj["steps"], int), "Trajectory missing 'steps' or not an int"
            model_stats = traj.get("model_stats")
            assert isinstance(model_stats, dict), "Trajectory 'model_stats' missing or not an object"
            # Validate model_stats keys and types (if present)
            for k, v in model_stats.items():
                assert isinstance(v, (int, float)), f"model_stats value for '{k}' not int or float"
    except (requests.RequestException, AssertionError) as e:
        raise AssertionError(f"Failed testing valid directory '{valid_dir}': {e}")

    # Test 404 error with nonexistent directory
    invalid_dir = "nonexistent_dir"
    params = {"dir": invalid_dir}
    try:
        response = requests.get(url, params=params, timeout=TIMEOUT)
        assert response.status_code == 404, f"Expected 404 Not Found for missing directory, got {response.status_code}"
        # Optionally check response content for directory missing error message (string)
        error_msg = response.text
        assert isinstance(error_msg, str) and len(error_msg) > 0, "Expected error message string on 404 response"
    except (requests.RequestException, AssertionError) as e:
        raise AssertionError(f"Failed testing nonexistent directory '{invalid_dir}': {e}")

test_get_api_trajectories_list_with_metadata()