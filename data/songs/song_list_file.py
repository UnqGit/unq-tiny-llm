from util import get_create_artist_dir, artist_names, get_file_name
import json


def create_artist_list(path):
    with open(path / "list.json", "r", encoding="utf-8") as json_file:
        data = json.load(json_file)

    song_lyrics_created = {
        title: get_file_name(title)
        for title, metadata in data.items()
        if metadata["done"]
    }
    
    with open(path / "songlist.json", "w", encoding="utf-8") as json_file:
        json.dump(song_lyrics_created, json_file, indent=4)


def main():
    artists_dir = [get_create_artist_dir(artist_name) for artist_name in artist_names()]
    for artist_dir in artists_dir:
        create_artist_list(artist_dir)


if __name__ == '__main__':
    main()