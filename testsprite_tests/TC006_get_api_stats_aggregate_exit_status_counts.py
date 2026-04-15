import requests

BASE_URL = "http://localhost:5000"
TIMEOUT = 30


def test_get_api_stats_aggregate_exit_status_counts():
    # 1. Test GET /api/stats with valid directory parameter 'trajectories'
    url = f"{BASE_URL}/api/stats"
    params = {"dir": "trajectories"}

    try:
        response = requests.get(url, params=params, timeout=TIMEOUT)
        assert response.status_code == 200, f"Expected 200 but got {response.status_code}"
        data = response.json()
        # Validate keys
        expected_keys = {"directory", "total", "submitted", "forfeited", "errors", "step_limit_reached", "other"}
        assert expected_keys <= data.keys(), f"Response JSON keys missing expected keys: {expected_keys - data.keys()}"
        assert data["directory"] == "trajectories"
        # Validate count values are integers and logical total count >= sum of parts
        for key in expected_keys - {"directory"}:
            val = data[key]
            assert isinstance(val, int), f"{key} should be integer, got {type(val)}"
            assert val >= 0, f"{key} should be non-negative"
        # total should be >= sum of all exit count categories, or equal
        sum_parts = data["submitted"] + data["forfeited"] + data["errors"] + data["step_limit_reached"] + data["other"]
        assert data["total"] >= sum_parts, "total count less than sum of individual status counts"

    except requests.RequestException as e:
        assert False, f"Request to /api/stats failed: {e}"

    # 2. Test GET /api/stats without directory parameter
    try:
        response = requests.get(f"{BASE_URL}/api/stats", timeout=TIMEOUT)
        # Accept 200 or 404 per observed behavior
        assert response.status_code in (200, 404), f"Expected 200 or 404 for missing dir param but got {response.status_code}"
    except requests.RequestException as e:
        assert False, f"Request to /api/stats without dir failed: {e}"

    # 3. Test GET /api/stats with missing/nonexistent directory parameter
    missing_dir = "missing_dir_that_does_not_exist_for_testing"
    try:
        response = requests.get(url, params={"dir": missing_dir}, timeout=TIMEOUT)
        assert response.status_code == 404, f"Expected 404 for missing dir '{missing_dir}' but got {response.status_code}"
    except requests.RequestException as e:
        assert False, f"Request to /api/stats with missing dir failed: {e}"


test_get_api_stats_aggregate_exit_status_counts()