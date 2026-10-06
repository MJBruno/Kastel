// std/string.ks
//
// Utilitaires officiels de chaînes au-dessus du type natif `str`.
// API canonique Kastel 1.0 : noms et signatures explicites.

export func capitalize(text: str) -> str {
    if text.size() == 0 {
        return text;
    }

    return text.substring(0, 1).upper() + text.substring(1, text.size() - 1);
}

// Ajoute `fill` jusqu'à atteindre exactement `width` caractères.
// Une largeur négative ou une chaîne de remplissage vide est une erreur.
export func pad_left(text: str, width: int, fill: str) -> Result<str, str> {
    if width < 0 {
        return Err("pad_left: width doit être >= 0");
    }
    if fill.size() == 0 {
        if text.size() >= width {
            return Ok(text);
        }
        return Err("pad_left: fill ne peut pas être vide");
    }

    let result = text;
    while result.size() < width {
        let missing = width - result.size();
        let prefix = fill;
        if fill.size() > missing {
            prefix = fill.substring(0, missing);
        }
        result = prefix + result;
    }

    return Ok(result);
}

// Variante droite de pad_left().
export func pad_right(text: str, width: int, fill: str) -> Result<str, str> {
    if width < 0 {
        return Err("pad_right: width doit être >= 0");
    }
    if fill.size() == 0 {
        if text.size() >= width {
            return Ok(text);
        }
        return Err("pad_right: fill ne peut pas être vide");
    }

    let result = text;
    while result.size() < width {
        let missing = width - result.size();
        let suffix = fill;
        if fill.size() > missing {
            suffix = fill.substring(0, missing);
        }
        result = result + suffix;
    }

    return Ok(result);
}

export func is_palindrome(text: str) -> bool {
    let normalized = text.lower();
    return normalized == normalized.reverse();
}

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

export func char_at_opt(text: str, index: int) -> Option<str> {
    if index < 0 || index >= text.size() {
        return None;
    }
    return Some(text.substring(index, 1));
}

export func truncate(text: str, max_length: int, ellipsis: str) -> Result<str, str> {
    if max_length < 0 {
        return Err("truncate: max_length doit être >= 0");
    }
    if text.size() <= max_length {
        return Ok(text);
    }
    return Ok(text.substring(0, max_length) + ellipsis);
}

export func slug(text: str) -> str {
    let lowered = text.lower();
    let result = "";
    let previous_was_dash = true;

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

export func count_occurrences(text: str, needle: str) -> int {
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
