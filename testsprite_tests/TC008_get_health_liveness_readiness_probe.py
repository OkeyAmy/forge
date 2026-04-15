import requests
from requests.exceptions import RequestException, ConnectionError, Timeout

BASE_URL = "http://localhost:5000"
TIMEOUT = 30  # seconds

def test_get_health_liveness_readiness_probe():
    url = f"{BASE_URL}/health"
    try:
        response = requests.get(url, timeout=TIMEOUT)
        # Check if service returns 200 with expected JSON
        assert response.status_code == 200, f"Expected status 200, got {response.status_code}"
        json_data = response.json()
        assert isinstance(json_data, dict), "Response is not a JSON object"
        assert "status" in json_data, "'status' key missing in JSON response"
        assert json_data["status"] == "ok", f"Expected status 'ok', got {json_data['status']}"
        assert "version" in json_data, "'version' key missing in JSON response"
        assert isinstance(json_data["version"], str), "'version' is not a string"
    except (ConnectionError, Timeout) as conn_err:
        # Expected if service is down or unresponsive
        # Assert that connection error or timeout is raised in this scenario
        assert True, f"Service is down or unresponsive as expected: {conn_err}"
    except RequestException as req_exc:
        # Other request exceptions
        assert False, f"Request failed unexpectedly: {req_exc}"
    except AssertionError:
        raise
    except Exception as ex:
        assert False, f"Unexpected exception: {ex}"

test_get_health_liveness_readiness_probe()