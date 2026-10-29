from data.dataset.songs.util import get_request, PATH, get_create_artist_dir, get_file_name
from bs4 import BeautifulSoup
import json


@get_request
def get_lyrics(response):
    soup = BeautifulSoup(response.text, "html.parser")
    lyrics = soup.select_one("div.col-xs-12.col-lg-8.text-center > div:not([class])")

    if lyrics:
        return lyrics.get_text(separator="\n", strip=True)
    else:
        return None


def get_artist_lyrics(path, name):
    artist_dir = path.parent

    with open(path, "r", encoding="utf-8") as json_file:
        data = json.load(json_file)

    i = 0
    retain_step = 5
    for title, songdata in data.items():
        if songdata["done"] == False:
            lyrics = get_lyrics(songdata["link"])
            if lyrics:
                file_name = get_file_name(title)
                file_path = artist_dir / file_name
                
                with open(file_path, "w", encoding="utf-8") as lyrics_file:
                    lyrics_file.write(f"Song by: {name}\nTitle: {title}\n\nLyrics:\n{lyrics}")

                songdata["done"] = True
                i += 1

                if i == retain_step:
                    with open(path, "w", encoding="utf-8") as json_file:
                        json.dump(data, json_file, indent=4)
                    i = 0

            else:
                print(f"WARNING: could not fetch lyrics for song: {title}\n")

    with open(path, "w", encoding="utf-8") as json_file:
        json.dump(data, json_file, indent=4)


def main():
    with open(PATH / "artistpath.json", "r", encoding="utf-8") as json_file:
        data = json.load(json_file)
    created_artist_folders = [(get_create_artist_dir(d["name"]), d["name"]) for d in data]
    
    for f, name in created_artist_folders:
        list_path = f / "list.json"
        if list_path.is_file():
            get_artist_lyrics(list_path, name)
        else:
            print(f"WARNING: list does not exist for {f}")
        

if __name__ == '__main__':
    main()