// std/json.ks
// JSON officiel : parsing/encodage + accès fichier sous Result.

export func encode(value: dynamic) -> Result<str, str> {
    try {
        return Ok(json_encode(value));
    } catch (error) {
        return Err(error);
    }
}

export func decode(text: str) -> Result<dynamic, str> {
    try {
        return Ok(json_decode(text));
    } catch (error) {
        return Err(error);
    }
}

export func read_file(path: str) -> Result<dynamic, str> {
    try {
        return Ok(json_decode(file_read(path)));
    } catch (error) {
        return Err(error);
    }
}

export func write_file(path: str, value: dynamic) -> Result<bool, str> {
    match encode(value) {
        Ok(text) => {
            try {
                file_write(path, text);
                return Ok(true);
            } catch (error) {
                return Err(error);
            }
        }
        Err(error) => {
            return Err(error);
        }
    }
}

export func get_path(data: dynamic, path: str) -> Option<dynamic> {
    let current = data;
    for name in path.split(".") {
        if type(current) != "dict" || !current.contains(name) {
            return None;
        }
        current = current.get(name);
    }
    return Some(current);
}
