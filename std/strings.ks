// std/strings.ks
//
// Utilitaires de chaînes de caractères au-dessus des méthodes
// natives de string (upper, lower, length, substring, ...).
//
// Usage : from std.strings import capitalize, is_palindrome;

export func capitalize(text) {
    if text.length == 0 {
        return text;
    }

    return text.substring(0, 1).upper() + text.substring(1, text.length - 1);
}

export func pad_left(text, width, fill) {
    let result = text;

    while result.length < width {
        result = fill + result;
    }

    return result;
}

export func pad_right(text, width, fill) {
    let result = text;

    while result.length < width {
        result = result + fill;
    }

    return result;
}

export func is_palindrome(text) {
    let normalized = text.lower();
    let length = normalized.length;
    let half = floor(length / 2);
    let i = 0;

    while i < half {
        if normalized.char_at(i) != normalized.char_at(length - 1 - i) {
            return false;
        }

        i = i + 1;
    }

    return true;
}

// Nombre d'occurrences NON chevauchantes de `needle` dans `text`.
export func count_occurrences(text, needle) {
    if needle.length == 0 {
        return 0;
    }

    let count = 0;
    let i = 0;
    let limit = text.length - needle.length;

    while i <= limit {
        if text.substring(i, needle.length) == needle {
            count = count + 1;
            i = i + needle.length;
        } else {
            i = i + 1;
        }
    }

    return count;
}
