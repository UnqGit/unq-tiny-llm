use std::fs::File;
use std::io::Read;
use std::path::Path;

fn load_file(path: &Path) -> Result<String, String> {
    let Some(ext) = path.extension() else {
        return Err(format!("File `{}` should be a text `.txt` file.\nNo extensions to resolve file type.", path.display()));
    };
    let Some(extension) = ext.to_str() else {
        return Err(format!("Could not read the extension of file `{}`", path.display()));
    };
    if extension != "txt" {
        return Err(format!("File `{}` should be a text `.txt` file.\nFound extension {:#?}", path.display(), extension));
    }

    let mut file = File::open(path)
        .map_err(|e| format!("Could not open file `{}`:\n{}", path.display(), e))?;

    let mut content: String = String::new();
    file.read_to_string(&mut content)
        .map_err(|s| format!("Could not read the file `{}` as string:\n{}", path.display(), s))?;

    Ok(content)
}

pub fn load_data(path: impl AsRef<Path>) -> Result<String, String> {
    let path_ref: &Path = path.as_ref();

    if !path_ref.exists() {
        return Err(format!("Path `{}` does not exist and could not be opened.", path_ref.display()));
    }

    if path_ref.is_file() {
        return load_file(path_ref);
    }

    Ok(String::new())
}