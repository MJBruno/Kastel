// std/fs.ks
// API officielle des fichiers. Les erreurs récupérables sont des Result.

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

export func write_text(path: str, content: str) -> Result<bool, str> {
    try {
        file_write(path, content);
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
