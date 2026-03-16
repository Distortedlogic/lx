import json
import os
import sys
from typing import Optional

def load_config(path: str) -> dict:
    with open(path) as f:
        data = json.load(f)
    return data

def get_user(data: dict, name: str) -> Optional[str]:
    result = data.get("users", {}).get(name, "")
    return result if result else None

def process_items(items: list) -> list:
    total = sum([x["value"] for x in items])
    names = [x["name"] for x in items]
    filtered = list(filter(lambda x: x["value"] > 0, items))
    return {"total": total, "names": names, "filtered": filtered}

def get_status(code):
    if code == "active":
        return True
    elif code == "inactive":
        return False
    elif code == "pending":
        return None
    return None

class DataStore:
    def __init__(self):
        self.data = {}

def store_item(store: DataStore, key: str, value):
    store.data[key] = value

def fetch_item(store: DataStore, key: str):
    return store.data.get(key, None)

def run_pipeline(items=[]):
    results = []
    for item in items:
        try:
            val = int(item)
            results.append(val)
        except:
            pass
    return results

if __name__ == "__main__":
    config = load_config("config.json")
    status = get_status("active")
    print(f"Status: {status}")
