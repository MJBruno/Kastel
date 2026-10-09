// std/fs.ks
// API officielle du système de fichiers. Les erreurs récupérables sont des Result.

export type FileMetadata = {
    size: int,
    is_file: bool,
    is_dir: bool,
    is_symlink: bool,
    readonly: bool
};

export func read_text(path: str) -> Result<str, str> {
    try {
        return Ok(file_read(path));
    } catch (error) {
        return Err(error);
    }
}

export func read_lines(path: str) -> Result<List<str>, str> {
    try {
        return Ok(file_read_lines(path));
    } catch (error) {
        return Err(error);
    }
}

export func read_bytes(path: str) -> Result<List<int>, str> {
    try {
        return Ok(file_read_bytes(path));
    } catch (error) {
        return Err(error);
    }
}

export func write_text(path: str, content: str) -> Result<bool, str> {
    try {
        file_write(path, content);
        return Ok(true);
    } catch (error) {
        return Err(error);
    }
}

export func write_bytes(path: str, data: List<int>) -> Result<bool, str> {
    try {
        file_write_bytes(path, data);
        return Ok(true);
    } catch (error) {
        return Err(error);
    }
}

export func append_text(path: str, content: str) -> Result<bool, str> {
    try {
        file_append(path, content);
        return Ok(true);
    } catch (error) {
        return Err(error);
    }
}

export func delete_file(path: str) -> Result<bool, str> {
    try {
        file_delete(path);
        return Ok(true);
    } catch (error) {
        return Err(error);
    }
}

export func size_of(path: str) -> Result<int, str> {
    try {
        return Ok(file_size(path));
    } catch (error) {
        return Err(error);
    }
}

export func copy_file(source: str, destination: str) -> Result<bool, str> {
    try {
        file_copy(source, destination);
        return Ok(true);
    } catch (error) {
        return Err(error);
    }
}

export func rename(source: str, destination: str) -> Result<bool, str> {
    try {
        file_rename(source, destination);
        return Ok(true);
    } catch (error) {
        return Err(error);
    }
}

export func canonicalize(path: str) -> Result<str, str> {
    try {
        return Ok(file_canonicalize(path));
    } catch (error) {
        return Err(error);
    }
}

export func metadata(path: str) -> Result<FileMetadata, str> {
    try {
        return Ok(file_metadata(path));
    } catch (error) {
        return Err(error);
    }
}

export func create_dir(path: str) -> Result<bool, str> {
    try {
        file_create_dir(path);
        return Ok(true);
    } catch (error) {
        return Err(error);
    }
}

export func create_dir_all(path: str) -> Result<bool, str> {
    try {
        file_create_dir_all(path);
        return Ok(true);
    } catch (error) {
        return Err(error);
    }
}

export func remove_dir(path: str) -> Result<bool, str> {
    try {
        file_remove_dir(path);
        return Ok(true);
    } catch (error) {
        return Err(error);
    }
}

export func remove_dir_all(path: str) -> Result<bool, str> {
    try {
        file_remove_dir_all(path);
        return Ok(true);
    } catch (error) {
        return Err(error);
    }
}

export func read_dir(path: str) -> Result<List<str>, str> {
    try {
        return Ok(file_read_dir(path));
    } catch (error) {
        return Err(error);
    }
}

export func exists(path: str) -> bool {
    return file_exists(path);
}

export func read_text_or(path: str, fallback: str) -> str {
    match read_text(path) {
        Ok(content) => {
            return content;
        }
        Err(_) => {
            return fallback;
        }
    }
}
