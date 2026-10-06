// std/regex.ks
// Expressions régulières Unicode, API officielle de Kastel.
// Les erreurs de compilation de motif sont toujours représentées par Result.

export type RegexMatch = {
    start: int,
    end: int,
    text: str,
    groups: List<Option<str>>
};

export func is_match(pattern: str, text: str) -> Result<bool, str> {
    return regex_is_match(pattern, text);
}

export func find(pattern: str, text: str) -> Result<Option<RegexMatch>, str> {
    return regex_find(pattern, text);
}

export func find_all(pattern: str, text: str) -> Result<List<RegexMatch>, str> {
    return regex_find_all(pattern, text);
}

export func replace(pattern: str, text: str, replacement: str) -> Result<str, str> {
    return regex_replace(pattern, text, replacement);
}

export func split(pattern: str, text: str) -> Result<List<str>, str> {
    return regex_split(pattern, text);
}

export func escape(text: str) -> str {
    return regex_escape(text);
}
