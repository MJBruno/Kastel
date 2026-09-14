//! Utilitaires partagés pour la manipulation de texte et de positions.
//!
//! Convention :
//! - `character` reçu de VS Code : unités UTF-16 (LSP) ;
//! - offsets internes (spans, occurrences) : bytes UTF-8 ;
//! - `utf16_character_to_byte_index` garantit toujours une frontière
//!   de char UTF-8, donc `text[index..]` ne panique jamais.

/// Convertit une position `character` LSP (en unités UTF-16) en index byte.
pub fn utf16_character_to_byte_index(text: &str, character: usize) -> usize {
    if character == 0 {
        return 0;
    }

    let mut utf16_units = 0usize;

    for (byte_index, ch) in text.char_indices() {
        let width = ch.len_utf16();

        if utf16_units + width > character {
            return byte_index;
        }

        utf16_units += width;

        if utf16_units == character {
            return byte_index + ch.len_utf8();
        }
    }

    text.len()
}

/// Vrai si le byte peut faire partie d'un identifiant Kastel.
pub fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// Vrai si le char peut faire partie d'un identifiant Kastel.
pub fn is_identifier_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Renvoie l'identifiant situé à la position `character` (UTF-16) sur `line`.
///
/// `line` est un index de ligne 0-based (comme LSP).
/// `character` est en unités UTF-16 (comme LSP).
pub fn find_word_at(source: &str, line: usize, character: usize) -> Option<&str> {
    let line_text = source.lines().nth(line)?;

    let byte_index = utf16_character_to_byte_index(line_text, character);

    let bytes = line_text.as_bytes();

    let mut start = byte_index;

    let mut end = byte_index;

    while start > 0 && is_identifier_byte(bytes[start - 1]) {
        start -= 1;
    }

    while end < bytes.len() && is_identifier_byte(bytes[end]) {
        end += 1;
    }

    if start == end {
        return None;
    }

    Some(&line_text[start..end])
}

/// Vrai si `[start, end)` dans `source` est bordé par des non-identifiants.
pub fn is_identifier_boundary(source: &str, start: usize, end: usize) -> bool {
    let before = source[..start].chars().next_back();

    let after = source[end..].chars().next();

    !before.is_some_and(is_identifier_char) && !after.is_some_and(is_identifier_char)
}

/// Trouve tous les offsets byte des occurrences de `name` en tant
/// qu'identifiant entier (ignore `VALUE2` quand on cherche `VALUE`).
pub fn find_identifier_occurrences(source: &str, name: &str) -> Vec<usize> {
    let bytes = source.as_bytes();

    let name_bytes = name.as_bytes();

    let mut occurrences = Vec::new();

    if name_bytes.is_empty() {
        return occurrences;
    }

    let mut offset = 0;

    while offset + name_bytes.len() <= bytes.len() {
        if &bytes[offset..offset + name_bytes.len()] == name_bytes
            && is_identifier_boundary(source, offset, offset + name_bytes.len())
        {
            occurrences.push(offset);

            offset += name_bytes.len();
        } else {
            offset += 1;
        }
    }

    occurrences
}

/// Renvoie le préfixe d'identifiant se terminant à `byte_index` sur `line`.
///
/// `byte_index` doit être une frontière de char UTF-8
/// (ce que garantit `utf16_character_to_byte_index`).
pub fn current_prefix(line: &str, byte_index: usize) -> &str {
    let end = byte_index.min(line.len());

    let bytes = line.as_bytes();

    let mut start = end;

    while start > 0 {
        let byte = bytes[start - 1];

        if is_identifier_byte(byte) {
            start -= 1;
        } else {
            break;
        }
    }

    &line[start..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf16_conversion_handles_ascii() {
        assert_eq!(utf16_character_to_byte_index("hello", 0,), 0);

        assert_eq!(utf16_character_to_byte_index("hello", 3,), 3);
    }

    #[test]
    fn utf16_conversion_handles_surrogate_pair() {
        let source = "😀abc";

        // 1 unité UTF-16 = début de l'emoji
        assert_eq!(utf16_character_to_byte_index(source, 1,), 0);

        // 2 unités = après l'emoji
        assert_eq!(utf16_character_to_byte_index(source, 2,), 4);

        // 3 unités = après 'a'
        assert_eq!(utf16_character_to_byte_index(source, 3,), 5);
    }

    #[test]
    fn utf16_conversion_clamps_to_end() {
        assert_eq!(utf16_character_to_byte_index("abc", 100,), 3);
    }

    #[test]
    fn find_word_at_returns_identifier() {
        let source = "print(VALUE)\n";

        assert_eq!(find_word_at(source, 0, 7), Some("VALUE"));
    }

    #[test]
    fn find_word_at_handles_utf16_position() {
        // 😀 = 2 unités UTF-16 / 4 bytes.
        // character = 4 → au milieu de VALUE ("VA|LUE")
        let source = "😀VALUE\n";

        assert_eq!(find_word_at(source, 0, 4), Some("VALUE"));
    }

    #[test]
    fn find_word_at_returns_none_off_identifier() {
        let source = "a + b\n";

        // 0:'a' 1:' ' 2:'+' 3:' ' 4:'b'
        // position 2 : ni le byte avant (' ') ni le byte
        // sur place ('+') n'est un identifiant → None.
        assert_eq!(find_word_at(source, 0, 2), None);
    }

    #[test]
    fn find_word_at_returns_digits_as_identifier() {
        // Comportement actuel : les chiffres sont traités comme
        // des bytes d'identifiant. Le filtrage sémantique se fait
        // plus haut (lookup dans SymbolIndex).
        let source = "print(42)\n";

        assert_eq!(find_word_at(source, 0, 6), Some("42"));
    }

    #[test]
    fn find_word_at_does_not_panic_mid_char() {
        // character = 1 tombe au milieu de 😀 en UTF-8.
        // Doit renvoyer le mot adjacent sans paniquer.
        let source = "😀VALUE\n";

        let result = find_word_at(source, 0, 1);

        // Byte index = 0 (frontière), on remonte : rien avant.
        // On descend : 😀 n'est pas un identifiant → end = 0.
        // Donc None.
        assert_eq!(result, None);
    }

    #[test]
    fn finds_all_identifier_occurrences() {
        let source = "const VALUE = 42\n\
             print(VALUE)\n\
             VALUE = 43\n";

        let occurrences = find_identifier_occurrences(source, "VALUE");

        assert_eq!(occurrences.len(), 3);
    }

    #[test]
    fn ignores_partial_identifier_matches() {
        let source = "VALUE\n\
             VALUE2\n\
             MY_VALUE\n\
             VALUE\n";

        let occurrences = find_identifier_occurrences(source, "VALUE");

        assert_eq!(occurrences.len(), 2);
    }

    #[test]
    fn empty_name_returns_no_occurrence() {
        assert!(find_identifier_occurrences("abc", "",).is_empty());
    }

    #[test]
    fn current_prefix_extracts_identifier() {
        assert_eq!(current_prefix("print(VA", 8), "VA");
    }

    #[test]
    fn current_prefix_stops_at_non_identifier() {
        assert_eq!(current_prefix("a + bc", 6), "bc");
    }

    #[test]
    fn current_prefix_empty_when_no_identifier() {
        assert_eq!(current_prefix("a + ", 4), "");
    }
}
