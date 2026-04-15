import requests
import time

BASE_URL = "http://localhost:5000"
TIMEOUT = 30

def test_post_get_delete_api_watch_background_github_label_watcher():
    watch_url = f"{BASE_URL}/api/watch"
    headers = {"Content-Type": "application/json"}
    # Payload adjusted to only include documented fields
    watch_payload = {
        "repo": "owner/repo",
        "label": "forge",
        "interval": 60
    }
    
    # 1. Ensure no watcher is running by deleting any existing watcher (ignore failures)
    try:
        resp_clean = requests.delete(watch_url, timeout=TIMEOUT)
        if resp_clean.status_code not in (200, 404):
            resp_clean.raise_for_status()
    except requests.RequestException:
        pass
    
    # 2. POST /api/watch to start watcher
    try:
        resp_start = requests.post(watch_url, json=watch_payload, timeout=TIMEOUT)
        assert resp_start.status_code == 200, f"Expected 200 on start watcher, got {resp_start.status_code}"
        start_data = resp_start.json()
        assert start_data.get("running") is True, "Watcher running should be True after start"
        assert start_data.get("repo") == watch_payload["repo"], "Returned repo does not match"
        # label could be optional, but provided here
        assert start_data.get("label") == watch_payload["label"], "Returned label does not match"
        assert start_data.get("started_at") is not None, "started_at field should be a timestamp"
    except Exception as e:
        raise e

    # 3. POST /api/watch again with same or different params -> expect 409 conflict
    try:
        resp_conflict = requests.post(watch_url, json=watch_payload, timeout=TIMEOUT)
        assert resp_conflict.status_code == 409, f"Expected 409 conflict when starting watcher again, got {resp_conflict.status_code}"
        content_type = resp_conflict.headers.get('Content-Type', '')
        if 'application/json' in content_type:
            conflict_data = resp_conflict.json()
            # "message" indicating watch already running expected in body (not strictly confirmed in PRD but typical)
            assert "watch" in conflict_data.get("message", "").lower()
        else:
            # If not JSON, check raw text
            assert "watch" in resp_conflict.text.lower()
    except requests.HTTPError as e:
        raise e
    except Exception as e:
        raise e

    # 4. GET /api/watch to retrieve current watcher status
    try:
        resp_get = requests.get(watch_url, timeout=TIMEOUT)
        assert resp_get.status_code == 200, f"Expected 200 on get watcher, got {resp_get.status_code}"
        get_data = resp_get.json()
        assert get_data.get("running") is True, "Watcher running should be True while watcher active"
        assert get_data.get("repo") == watch_payload["repo"], "GET watcher repo mismatch"
        assert get_data.get("label") == watch_payload["label"], "GET watcher label mismatch"
        assert get_data.get("started_at") is not None, "GET watcher started_at should not be None"
    except Exception as e:
        raise e

    # 5. DELETE /api/watch to stop watcher and verify stopped status
    try:
        resp_del = requests.delete(watch_url, timeout=TIMEOUT)
        assert resp_del.status_code == 200, f"Expected 200 on delete watcher, got {resp_del.status_code}"
        del_data = resp_del.json()
        assert del_data.get("running") is False, "Watcher running should be False after stop"
        # The fields repo, label, started_at should be null or None
        assert del_data.get("repo") in (None, "") or del_data.get("repo") is None
        assert del_data.get("label") in (None, "") or del_data.get("label") is None
        assert del_data.get("started_at") in (None, "") or del_data.get("started_at") is None
    except Exception as e:
        raise e

test_post_get_delete_api_watch_background_github_label_watcher()
