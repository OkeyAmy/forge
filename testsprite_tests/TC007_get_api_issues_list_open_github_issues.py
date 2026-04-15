import requests

BASE_URL = "http://localhost:5000"
TIMEOUT = 30

def test_get_api_issues_list_open_github_issues():
    session = requests.Session()
    headers = {"Accept": "application/json"}

    # Test case 1: Valid repo parameter only
    params_valid = {"repo": "python/cpython"}
    try:
        resp = session.get(f"{BASE_URL}/api/issues", params=params_valid, headers=headers, timeout=TIMEOUT)
        assert resp.status_code == 200, f"Expected 200, got {resp.status_code}"
        data = resp.json()
        assert "repo" in data and data["repo"] == params_valid["repo"], "Response missing or incorrect repo"
        assert "count" in data and isinstance(data["count"], int), "Response missing or invalid count"
        assert "issues" in data and isinstance(data["issues"], list), "Response missing or invalid issues list"
        for issue in data["issues"]:
            assert isinstance(issue.get("number"), int), "Issue number missing or invalid"
            assert isinstance(issue.get("title"), str) and issue.get("title"), "Issue title missing or empty"
            assert isinstance(issue.get("url"), str) and issue.get("url").startswith("https://github.com/"), "Issue url invalid"
            assert isinstance(issue.get("labels"), list), "Issue labels missing or invalid"
        # Validate no pull requests present by ensuring issues only
        # (Assuming pull requests have "pull_request" field or excluded by server)
    except Exception as e:
        raise AssertionError(f"Valid repo GET /api/issues test failed: {e}")

    # Test case 2: Valid repo with label and limit parameters
    params_labeled = {"repo": "python/cpython", "label": "bug", "limit": 5}
    try:
        resp = session.get(f"{BASE_URL}/api/issues", params=params_labeled, headers=headers, timeout=TIMEOUT)
        assert resp.status_code == 200, f"Expected 200, got {resp.status_code}"
        data = resp.json()
        assert data.get("repo") == params_labeled["repo"], "Repo mismatch"
        assert isinstance(data.get("count"), int), "Count missing or invalid"
        issues = data.get("issues")
        assert isinstance(issues, list), "Issues field missing or invalid"
        # Count should be less or equal to limit
        assert data["count"] <= params_labeled["limit"], "Count exceeds limit"
        # Check label presence in issues
        for issue in issues:
            assert "labels" in issue and params_labeled["label"] in issue["labels"], "Issue missing specified label"
            assert isinstance(issue.get("number"), int), "Issue number invalid"
            assert isinstance(issue.get("title"), str) and issue.get("title"), "Issue title invalid"
            assert isinstance(issue.get("url"), str) and issue.get("url").startswith("https://github.com/"), "Issue url invalid"
    except Exception as e:
        raise AssertionError(f"Valid repo with label and limit GET /api/issues test failed: {e}")

    # Test case 3: Invalid repo triggers 502 error
    params_invalid = {"repo": "invalid/invalid-repo-that-does-not-exist-123456"}
    try:
        resp = session.get(f"{BASE_URL}/api/issues", params=params_invalid, headers=headers, timeout=TIMEOUT)
        assert resp.status_code == 502, f"Expected 502 for invalid repo, got {resp.status_code}"
        # Response body should contain error message string
        assert isinstance(resp.text, str) and len(resp.text) > 0, "502 response missing error message"
    except requests.exceptions.RequestException as re:
        raise AssertionError(f"Request exception during invalid repo 502 test: {re}")
    except Exception as e:
        raise AssertionError(f"Invalid repo GET /api/issues test failed: {e}")

test_get_api_issues_list_open_github_issues()