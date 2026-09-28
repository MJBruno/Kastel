// std/strings.ks
//
// Utilitaires de chaînes de caractères au-dessus des méthodes
// natives de string (upper, lower, length, substring, ...).
//
// Usage : from std.strings import capitalize, is_palindrome;

export func capitalize(text) {
    if text.size() == 0 {
        return text;
    }

    return text.substring(0, 1).upper() + text.substring(1, text.size() - 1);
}

export func pad_left(text, width, fill) {
    let result = text;

    while result.size() < width {
        result = fill + result;
    }

    return result;
}

export func pad_right(text, width, fill) {
    let result = text;

    while result.size() < width {
        result = result + fill;
    }

    return result;
}

// reverse() est une methode native de string : plus simple et plus
// sur qu'une boucle manuelle sur char_at() (pas de piege d'index).
export func is_palindrome(text) {
    let normalized = text.lower();
    return normalized == normalized.reverse();
}

// ------------------------------------------------------------------
// Conversions sures (Result) -- to_int()/to_float() natifs levent une
// exception peu parlante ("Operand must be numbers.") sur une chaine
// non numerique ; ces wrappers renvoient un message explicite.
// ------------------------------------------------------------------

export func parse_int(text: str) -> Result<int, str> {
    try {
        return Ok(text.to_int());
    } catch (_) {
        return Err("parse_int: '" + text + "' n'est pas un entier valide");
    }
}

export func parse_float(text: str) -> Result<float, str> {
    try {
        return Ok(text.to_float());
    } catch (_) {
        return Err("parse_float: '" + text + "' n'est pas un nombre valide");
    }
}

// ------------------------------------------------------------------
// Recherche -- index_of()/last_index_of() natifs renvoient -1 quand
// absent, sentinelle qu'on remonte ici en Option<int>.
// ------------------------------------------------------------------

export func find(text: str, needle: str) -> Option<int> {
    let index = text.index_of(needle);
    if index < 0 {
        return None;
    }
    return Some(index);
}

export func find_last(text: str, needle: str) -> Option<int> {
    let index = text.last_index_of(needle);
    if index < 0 {
        return None;
    }
    return Some(index);
}

// char_at() natif leve une exception hors bornes ; cette version
// renvoie None plutot que de forcer un try/catch pour un simple test.
export func char_at_opt(text: str, index: int) -> Option<str> {
    if index < 0 || index >= text.size() {
        return None;
    }
    return Some(text.substring(index, 1));
}

// Coupe `text` a `max_length` caracteres et ajoute `ellipsis` si une
// coupe a eu lieu (sinon `text` est renvoye tel quel, `ellipsis` non
// compte dans la limite).
export func truncate(text: str, max_length: int, ellipsis: str) -> str {
    if text.size() <= max_length {
        return text;
    }
    return text.substring(0, max_length) + ellipsis;
}

// Identifiant "slug" pratique pour une URL/un nom de fichier :
// minuscule, espaces et caracteres non alphanumeriques remplaces par
// un tiret, tirets consecutifs fusionnes, pas de tiret en bord.
export func slug(text: str) -> str {
    let lowered = text.lower();
    let result = "";
    let previous_was_dash = true; // evite un tiret en tete

    let i = 0;
    while i < lowered.size() {
        let ch = lowered.substring(i, 1);
        if ch.is_alphanumeric() {
            result = result + ch;
            previous_was_dash = false;
        } else if !previous_was_dash {
            result = result + "-";
            previous_was_dash = true;
        }
        i = i + 1;
    }

    if result.size() > 0 && result.substring(result.size() - 1, 1) == "-" {
        result = result.substring(0, result.size() - 1);
    }

    return result;
}

// Nombre d'occurrences NON chevauchantes de `needle` dans `text`.
export func count_occurrences(text, needle) {
    if needle.size() == 0 {
        return 0;
    }

    let count = 0;
    let i = 0;
    let limit = text.size() - needle.size();

    while i <= limit {
        if text.substring(i, needle.size()) == needle {
            count = count + 1;
            i = i + needle.size();
        } else {
            i = i + 1;
        }
    }

    return count;
}
