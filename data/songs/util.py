import requests
from functools import wraps
from urllib.parse import urljoin
from pathlib import Path
import time
import json


PATH = Path(__file__).resolve().parent
HEADERS = {"User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"}
# HEADERS = {
#     "User-Agent": (
#         "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
#         "AppleWebKit/537.36 (KHTML, like Gecko) "
#         "Chrome/131.0.0.0 Safari/537.36"
#     )
# }
ROOT_URL = "https://www.azlyrics.com"


def get_request(func):
    @wraps(func)
    def wrapper(url, *args, **kwargs):
        time.sleep(1)
        try:
            response = requests.get(url, headers=HEADERS, timeout=10, allow_redirects=True)
            response.raise_for_status()
            return func(response, *args, **kwargs)
        
        except requests.RequestException as error:
            print(f"Request Failed: {error}")
            return None
        
    return wrapper


def artist_names():
    with open(PATH / "artistpath.json", "r", encoding="utf-8") as json_file:
        data = json.load(json_file)

    return [d["name"] for d in data]


def get_file_name(name):
    return "".join(
        char for char in name if char.isalnum()
    ).lower() + ".txt"


def get_link(url):
    return urljoin(ROOT_URL + "/", url)


def get_create_artist_dir(name):
    artist_path = PATH / name.replace(" ", "").lower()
    artist_path.mkdir(parents=True, exist_ok=True)

    return artist_path