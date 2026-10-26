from util import get_request, get_link, get_create_artist_dir, PATH
from bs4 import BeautifulSoup
import json

@get_request
def get_song_dict(response):
    soup = BeautifulSoup(response.text, "html.parser")
    songs = soup.select("div.listalbum-item a")

    if songs:
        return {
            song.text.strip(): {
                "link": get_link(song.get("href")),
                "done": False
            }
            for song in songs
        }
    else:
        raise ValueError("No song list found")


def get_link_ext_using_name(name):
    if not name:
        raise ValueError("Name must exist to create the extension")

    prefix = "19" if name[0].isdigit() else name[0].lower()
    return f"/{prefix}/{name}.html"


def generate_song_lists(name, url):
    artist_path = get_create_artist_dir(name)
    song_dict = get_song_dict(url)

    if song_dict is None:
        raise RuntimeError(f"Could not fetch songs for {name}")
    
    with open(artist_path / "list.json", "w", encoding="utf-8") as json_file:
        json.dump(song_dict, json_file, indent=4)


def main():
    paths_file = PATH / "artistpath.json"
    with open(paths_file, "r", encoding="utf-8") as file:
        data = json.load(file)

    for d in data:
        if d["done"] == True:
            continue

        name = d["name"]
        namepath = d["path"]
        try:
            url = get_link(get_link_ext_using_name(namepath))
            generate_song_lists(name, url)
            d["done"] = True
        except Exception as error:
            print(f"Failed to process {name}: {error}")

    with open(paths_file, "w", encoding="utf-8") as file:
        json.dump(data, file, indent=4)

if __name__ == '__main__':
    main()