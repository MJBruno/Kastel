// std/json.ks
//
// Encodage/décodage JSON, natif (voir src/stdlib/json.rs — un vrai
// parseur, pas exprimable proprement en Kastel pur).
//
// Deux styles au choix :
//   - `encode` / `decode` / `read_file` / `write_file` : comportement
//     natif, lèvent une exception sur erreur (try/catch) ;
//   - `try_*` : mêmes opérations en Result<T, str>, à utiliser avec
//     `match` — recommandé pour toute entrée qui n'est pas de confiance
//     (fichier utilisateur, réseau).
//
// Usage :
//
//   import std.json;
//   match json.try_decode("{\"x\": 1}") {
//       Ok(data) => { println(data.get("x")); }
//       Err(message) => { println("JSON invalide: " + message); }
//   }

export const encode = json_encode;
export const decode = json_decode;

export func read_file(path) {
    return decode(file_read(path));
}

export func write_file(path, value) {
    file_write(path, encode(value));
}

export func try_encode(value) -> Result<str, str> {
    try {
        return Ok(json_encode(value));
    } catch (error) {
        return Err(error);
    }
}

export func try_decode(text: str) -> Result<any, str> {
    try {
        return Ok(json_decode(text));
    } catch (error) {
        return Err(error);
    }
}

export func try_read_file(path: str) -> Result<any, str> {
    try {
        return Ok(json_decode(file_read(path)));
    } catch (error) {
        return Err(error);
    }
}

export func try_write_file(path: str, value) -> Result<bool, str> {
    try {
        file_write(path, json_encode(value));
        return Ok(true);
    } catch (error) {
        return Err(error);
    }
}

// Lit un chemin "a.b.c" dans une valeur JSON décodée (Dict imbriqués),
// Option : None si un maillon manque ou n'est pas un dict.
export func get_path(data, path: str) -> Option<any> {
    let current = data;
    for name in path.split(".") {
        if type(current) != "dict" || !current.contains(name) {
            return None;
        }
        current = current.get(name);
    }
    return Some(current);
}
