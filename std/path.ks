// std/path.ks
// API officielle de manipulation des chemins.

export func join(segments: List<str>) -> str {
    return path_join(segments);
}

export func exists(path: str) -> bool {
    return path_exists(path);
}

export func is_dir(path: str) -> bool {
    return path_is_dir(path);
}

export func is_file(path: str) -> bool {
    return path_is_file(path);
}

export func absolute(path: str) -> Result<str, str> {
    try {
        return Ok(path_absolute(path));
    } catch (error) {
        return Err(error);
    }
}

export func basename(path: str) -> str {
    return path_basename(path);
}

export func dirname(path: str) -> str {
    return path_dirname(path);
}

export func extension(path: str) -> Option<str> {
    let value = path_extension(path);
    if value.size() == 0 {
        return None;
    }
    return Some(value);
}

export func stem(path: str) -> Option<str> {
    let value = path_stem(path);
    if value.size() == 0 {
        return None;
    }
    return Some(value);
}
