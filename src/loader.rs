use std::fs::{self, File, DirEntry};
use std::io::{Read};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataLoadResult {
    pub contents: Vec<String>,
    pub errors: Vec<String>,
}

fn load_file_data(path: &Path) -> Result<String, String> {
    let Some(extension) = path.extension() else {
        return Err(format!("File `{}` should be a text `.txt` file.\nCould not resolve extension for path.", path.display()));
    };
    if !extension.eq_ignore_ascii_case("txt") {
        return Err(format!("File `{}` should be a text `.txt` file.\nFound extension {:#?}", path.display(), extension));
    }

    let mut file = File::open(path)
        .map_err(|e| format!("Could not open file `{}`:\n{}", path.display(), e))?;

    let mut content: String = String::new();
    file.read_to_string(&mut content)
        .map_err(|s| format!("Could not read the file `{}` as string:\n{}", path.display(), s))?;

    Ok(content)
}

fn load_dir_data(dirpath: &Path) -> DataLoadResult {
    let mut load_result = DataLoadResult { contents: Vec::new(), errors: Vec::new() };
    let read_dir = match dirpath.read_dir() {
        Ok(dir) => dir,
        Err(err) => {
            load_result.errors.push(format!("read_dir call for path `{}` failed:{}", dirpath.display(), err));
            return load_result;
        }
    };
    let mut entries: Vec<DirEntry> = read_dir
        .filter_map(|entry| match entry {
            Ok(entries) => Some(entries),
            Err(err) => {
                load_result.errors.push(format!("Could not fetch entry for directory `{}`:\n{}", dirpath.display(), err));
                None
            }

        }).collect();

    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let entry_path = entry.path();
        let metadata_query = fs::symlink_metadata(&entry_path);
        let metadata = match metadata_query {
            Ok(md) => md,
            Err(e) => {
                load_result.errors.push(format!("Error reading meta data for {}: {}", entry_path.display(), e));
                continue;
            }
        };
        let file_type = metadata.file_type();
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_file() && entry_path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("txt")) {
            match load_file_data(&entry_path) {
                Ok(content) => load_result.contents.push(content),
                Err(err) => load_result.errors.push(err),
            }
        }
        else if file_type.is_dir() {
            let inner_load_result = load_dir_data(&entry_path);
            load_result.contents.extend(inner_load_result.contents);
            load_result.errors.extend(inner_load_result.errors);
        }
    }
    load_result
}

pub fn load_data(path: impl AsRef<Path>) -> Result<DataLoadResult, String> {
    let path_ref: &Path = path.as_ref();
    if !path_ref.exists() {
        return Err(format!("Path `{}` does not exist and could not be opened.", path_ref.display()));
    }

    let metadata_query = fs::metadata(path_ref);
    let metadata = match metadata_query {
        Ok(md) => md,
        Err(e) => return Err(format!("Error reading meta data for {}: {}", path_ref.display(), e)),
    };

    let path_type = metadata.file_type();
    if path_type.is_file() {
        let result = match load_file_data(path_ref) {
            Ok(content) => DataLoadResult {contents: vec![content], errors: Vec::new() },
            Err(err) => DataLoadResult {contents: Vec::new(), errors: vec![err] },
        };
        return Ok(result);
    }
    else if path_type.is_dir() {
        return Ok(load_dir_data(path_ref));
    }
    else {
        return Err(format!("Expected path `{}` to be either a file or a directory, got.", path_ref.display()));
    }
}