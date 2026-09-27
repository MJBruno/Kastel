// std/fs.ks
//
// Les natives fichier (file_read, file_write, ...) lèvent une
// exception native (`throw`) en cas d'erreur système, plutôt que de
// renvoyer un Result. Ce module les enveloppe systématiquement pour
// exposer une API Result<T, str> idiomatique, utilisable avec `match`
// sans jamais planter le programme appelant sur une erreur d'IO.

// ------------------------------------------------------------------
// Fichiers
// ------------------------------------------------------------------

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

export func file_size_of(path: str) -> Result<int, str> {
    try {
        return Ok(file_size(path));
    } catch (error) {
        return Err(error);
    }
}

// file_exists() natif ne lève jamais : simple passe-plat, gardé ici
// pour que tout le module fs.* soit auto-suffisant.
export func exists(path: str) -> bool {
    return file_exists(path);
}

// Lit un fichier texte, ou une valeur de repli s'il n'existe pas ou
// n'est pas lisible — pratique pour un fichier de config optionnel.
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

// ------------------------------------------------------------------
// JSON (au-dessus des fichiers)
// ------------------------------------------------------------------

export func read_json(path: str) -> Result<any, str> {
    try {
        return Ok(json_decode(file_read(path)));
    } catch (error) {
        return Err(error);
    }
}

export func write_json(path: str, value: any) -> Result<bool, str> {
    try {
        file_write(path, json_encode(value));
        return Ok(true);
    } catch (error) {
        return Err(error);
    }
}

// ------------------------------------------------------------------
// Chemins (path_* ne lève jamais, sauf path_absolute)
// ------------------------------------------------------------------

export func join(segments: List<str>) -> str {
    return path_join(segments);
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

// path_extension()/path_stem()/path_dirname() natifs renvoient "" en
// l'absence d'extension/racine/parent, ce qui est ambigu avec une
// vraie chaîne vide : on remonte ça en Option<str>.
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

export func dirname(path: str) -> Option<str> {
    let value = path_dirname(path);
    if value.size() == 0 {
        return None;
    }
    return Some(value);
}
