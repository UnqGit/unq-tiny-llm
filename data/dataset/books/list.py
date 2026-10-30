from pathlib import Path
import json

PATH = Path(__file__).resolve().parent

def text_file_list(dirpath: Path):
    return [
        path.name
        for path in dirpath.iterdir()
        if path.is_file() and path.suffix == '.txt'
    ]

def create_list_text_files_dir(dirpath: Path):
    list_path = dirpath / "book.json"
    file_list = text_file_list(dirpath)

    if len(file_list) > 0:
        with open(list_path, "w") as json_file:
            json.dump(file_list, json_file, indent=2)

    for path in dirpath.iterdir():
        if path.is_dir():
            create_list_text_files_dir(path)

def main():
    create_list_text_files_dir(PATH)

if __name__ == '__main__':
    main()