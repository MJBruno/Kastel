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

export func is_absolute(path: str) -> bool {
    return path_is_absolute(path);
}

export func is_relative(path: str) -> bool {
    return path_is_relative(path);
}

export func has_root(path: str) -> bool {
    return path_has_root(path);
}

export func starts_with(path: str, base: str) -> bool {
    return path_starts_with(path, base);
}

export func ends_with(path: str, suffix: str) -> bool {
    return path_ends_with(path, suffix);
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

export func strip_prefix(path: str, prefix: str) -> Result<str, str> {
    try {
        return Ok(path_strip_prefix(path, prefix));
    } catch (error) {
        return Err(error);
    }
}

export func with_extension(path: str, extension: str) -> str {
    return path_with_extension(path, extension);
}

export func with_file_name(path: str, name: str) -> str {
    return path_with_file_name(path, name);
}
