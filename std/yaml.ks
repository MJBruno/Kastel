// std/yaml.ks
//
// Sous-ensemble volontairement LIMITÉ de YAML, en Kastel pur (YAML
// complet — ancres, tags, scalaires multi-lignes, clés complexes — est
// une spécification énorme, hors de portée d'un parseur de cette taille
// sans risque de mal interpréter silencieusement). SUPPORTÉ :
//   - mappings `clé: valeur` et séquences `- item`, imbriqués par
//     indentation (espaces uniquement ; une tabulation est une erreur) ;
//   - un mapping comme élément de séquence : `- nom: x` puis `  age: 3` ;
//   - scalaires : chaînes nues ou entre "..." / '...', entiers,
//     flottants, true/false, null / ~ / vide ;
//   - `[]` et `{}` vides, et séquences inline de scalaires `[1, 2, 3]` ;
//   - commentaires `# ...` et séparateur de document `---` ignoré.
// NON SUPPORTÉ (Err explicite ou comportement documenté ici) :
//   ancres/alias (&x, *x), tags (!!x), scalaires multi-lignes (| >),
//   mappings inline non vides {a: 1}, plusieurs documents, clés non
//   textuelles.
//
// Usage :
//
//   import std.yaml;
//   match yaml.parse("nom: Ada\nlangages:\n  - kastel\n  - rust\n") {
//       Ok(data) => { println(data.get("langages")); }
//       Err(message) => { println(message); }
//   }

// Retire un commentaire de fin de ligne (hors chaines). Un `#` n'est un
// commentaire que s'il est en debut de ligne ou precede d'un espace.
func strip_comment(line: str) -> str {
    let quote = "";
    let i = 0;
    while i < line.size() {
        let ch = line.substring(i, 1);
        if quote != "" {
            if ch == "\\" && quote == "\"" {
                i = i + 1;
            } else if ch == quote {
                quote = "";
            }
        } else if ch == "\"" || ch == "'" {
            quote = ch;
        } else if ch == "#" && (i == 0 || line.substring(i - 1, 1) == " ") {
            return line.substring(0, i);
        }
        i = i + 1;
    }
    return line;
}

func unquote_double(text: str) -> Result<str, str> {
    let inner = text.substring(1, text.size() - 2);
    let result = "";
    let i = 0;
    while i < inner.size() {
        let ch = inner.substring(i, 1);
        if ch == "\\" {
            if i + 1 >= inner.size() {
                return Err("yaml: echappement incomplet dans " + text);
            }
            i = i + 1;
            let next = inner.substring(i, 1);
            if next == "n" {
                result = result + "\n";
            } else if next == "t" {
                result = result + "\t";
            } else if next == "\"" || next == "\\" {
                result = result + next;
            } else {
                return Err("yaml: echappement non supporte \\" + next + " dans " + text);
            }
        } else {
            result = result + ch;
        }
        i = i + 1;
    }
    return Ok(result);
}

func parse_scalar(raw: str) -> Result<any, str> {
    let text = raw.trim();

    if text == "" || text == "null" || text == "~" || text == "Null" || text == "NULL" {
        return Ok(None);
    }
    if text == "true" || text == "True" || text == "TRUE" {
        return Ok(true);
    }
    if text == "false" || text == "False" || text == "FALSE" {
        return Ok(false);
    }

    let first = text.substring(0, 1);

    if first == "&" || first == "*" || first == "!" {
        return Err("yaml: ancres, alias et tags non supportes: " + text);
    }
    if first == "|" || first == ">" {
        return Err("yaml: scalaires multi-lignes non supportes");
    }

    if first == "\"" {
        if text.size() < 2 || text.substring(text.size() - 1, 1) != "\"" {
            return Err("yaml: chaine non terminee: " + text);
        }
        return unquote_double(text);
    }
    if first == "'" {
        if text.size() < 2 || text.substring(text.size() - 1, 1) != "'" {
            return Err("yaml: chaine non terminee: " + text);
        }
        return Ok(text.substring(1, text.size() - 2).replace_all("''", "'"));
    }

    if text == "[]" {
        return Ok([]);
    }
    if text == "{}" {
        return Ok({});
    }
    if first == "[" {
        if text.substring(text.size() - 1, 1) != "]" {
            return Err("yaml: sequence inline non terminee: " + text);
        }
        let items = [];
        let inner = text.substring(1, text.size() - 2).trim();
        if inner.size() > 0 {
            for part in inner.split(",") {
                match parse_scalar(part) {
                    Ok(item) => {
                        items.add(item);
                    }
                    Err(message) => {
                        return Err(message);
                    }
                }
            }
        }
        return Ok(items);
    }
    if first == "{" {
        return Err("yaml: mappings inline non vides non supportes");
    }

    // Nombre ? (sinon chaine nue)
    let looks_numeric = false;
    let i = 0;
    while i < text.size() {
        let ch = text.substring(i, 1);
        if ch.is_digit() {
            looks_numeric = true;
        } else if !(ch == "-" || ch == "+" || ch == "." || ch == "e" || ch == "E" || ch == "_") {
            looks_numeric = false;
            break;
        }
        i = i + 1;
    }
    if looks_numeric {
        let number = text.replace_all("_", "");
        try {
            if number.contains(".") || number.contains("e") || number.contains("E") {
                return Ok(number.to_float());
            }
            return Ok(number.to_int());
        } catch (_) {
            return Ok(text);
        }
    }

    return Ok(text);
}

// Cherche le `:` separateur cle/valeur (suivi d'un espace ou en fin de
// ligne, hors guillemets). -1 si absent.
func find_key_separator(content: str) -> int {
    let quote = "";
    let i = 0;
    while i < content.size() {
        let ch = content.substring(i, 1);
        if quote != "" {
            if ch == "\\" && quote == "\"" {
                i = i + 1;
            } else if ch == quote {
                quote = "";
            }
        } else if ch == "\"" || ch == "'" {
            quote = ch;
        } else if ch == ":" && (i + 1 == content.size() || content.substring(i + 1, 1) == " ") {
            return i;
        }
        i = i + 1;
    }
    return -1;
}

func is_sequence_item(content: str) -> bool {
    return content == "-" || content.starts_with("- ");
}

// lines : List de [indent, contenu, numero_de_ligne]. Renvoie Ok([valeur, prochain_index]).
func parse_block(lines, start: int, indent: int) -> Result<any, str> {
    if is_sequence_item(lines[start][1]) {
        return parse_sequence(lines, start, indent);
    }
    return parse_mapping(lines, start, indent);
}

func parse_sequence(lines, start: int, indent: int) -> Result<any, str> {
    let items = [];
    let index = start;

    while index < lines.size() && lines[index][0] == indent && is_sequence_item(lines[index][1]) {
        let content = lines[index][1];
        let rest = content.substring(1, content.size() - 1);
        let stripped = rest.trim();
        let line_no = lines[index][2];

        if stripped == "" {
            // "-" seul : bloc imbrique a la ligne suivante, sinon null.
            if index + 1 < lines.size() && lines[index + 1][0] > indent {
                match parse_block(lines, index + 1, lines[index + 1][0]) {
                    Ok(pair) => {
                        items.add(pair[0]);
                        index = pair[1];
                    }
                    Err(message) => {
                        return Err(message);
                    }
                }
            } else {
                items.add(None);
                index = index + 1;
            }
        } else if find_key_separator(stripped) >= 0 && !stripped.starts_with("\"") && !stripped.starts_with("'") {
            // "- cle: valeur" : on reecrit la ligne comme le debut d'un
            // mapping decale, puis on laisse parse_mapping continuer.
            let offset = indent + 1 + (rest.size() - rest.trim_start().size());
            lines[index] = [offset, stripped, line_no];
            match parse_mapping(lines, index, offset) {
                Ok(pair) => {
                    items.add(pair[0]);
                    index = pair[1];
                }
                Err(message) => {
                    return Err(message);
                }
            }
        } else {
            match parse_scalar(stripped) {
                Ok(value) => {
                    items.add(value);
                    index = index + 1;
                }
                Err(message) => {
                    return Err(message + " (ligne " + str(line_no) + ")");
                }
            }
        }
    }

    return Ok([items, index]);
}

func parse_mapping(lines, start: int, indent: int) -> Result<any, str> {
    let mapping = {};
    let index = start;

    while index < lines.size() && lines[index][0] == indent {
        let content = lines[index][1];
        let line_no = lines[index][2];

        if is_sequence_item(content) {
            return Err("yaml ligne " + str(line_no) + ": element de sequence inattendu dans un mapping");
        }

        let colon = find_key_separator(content);
        if colon < 0 {
            return Err("yaml ligne " + str(line_no) + ": ':' attendu apres la cle");
        }

        let key_text = content.substring(0, colon).trim();
        let key = key_text;
        if key_text.starts_with("\"") {
            match unquote_double(key_text) {
                Ok(unquoted) => {
                    key = unquoted;
                }
                Err(message) => {
                    return Err("yaml ligne " + str(line_no) + ": " + message);
                }
            }
        } else if key_text.starts_with("'") && key_text.ends_with("'") && key_text.size() >= 2 {
            key = key_text.substring(1, key_text.size() - 2);
        }
        if key == "" {
            return Err("yaml ligne " + str(line_no) + ": cle vide");
        }
        if mapping.contains(key) {
            return Err("yaml ligne " + str(line_no) + ": cle en double '" + key + "'");
        }

        let value_text = content.substring(colon + 1, content.size() - colon - 1).trim();

        if value_text == "" {
            // Bloc imbrique (plus indente, ou sequence au meme niveau).
            let next = index + 1;
            if next < lines.size() && lines[next][0] > indent {
                match parse_block(lines, next, lines[next][0]) {
                    Ok(pair) => {
                        mapping.set(key, pair[0]);
                        index = pair[1];
                    }
                    Err(message) => {
                        return Err(message);
                    }
                }
            } else if next < lines.size() && lines[next][0] == indent && is_sequence_item(lines[next][1]) {
                match parse_sequence(lines, next, indent) {
                    Ok(pair) => {
                        mapping.set(key, pair[0]);
                        index = pair[1];
                    }
                    Err(message) => {
                        return Err(message);
                    }
                }
            } else {
                mapping.set(key, None);
                index = index + 1;
            }
        } else {
            match parse_scalar(value_text) {
                Ok(value) => {
                    mapping.set(key, value);
                    index = index + 1;
                }
                Err(message) => {
                    return Err(message + " (ligne " + str(line_no) + ")");
                }
            }
        }
    }

    return Ok([mapping, index]);
}

export func parse(text: str) -> Result<any, str> {
    let lines = [];
    let number = 0;

    for raw_line in text.split("\n") {
        number = number + 1;
        let cleaned = strip_comment(raw_line.trim_end());
        if cleaned.trim() == "" || cleaned.trim() == "---" {
            continue;
        }
        if cleaned.contains("\t") && cleaned.substring(0, 1) == "\t" {
            return Err("yaml ligne " + str(number) + ": tabulation dans l'indentation (espaces uniquement)");
        }
        let content = cleaned.trim_start();
        let indent = cleaned.size() - content.size();
        lines.add([indent, content.trim_end(), number]);
    }

    if lines.is_empty() {
        return Ok(None);
    }

    // Un document reduit a un seul scalaire ("42", "bonjour").
    if lines.size() == 1 && !is_sequence_item(lines[0][1]) && find_key_separator(lines[0][1]) < 0 {
        return parse_scalar(lines[0][1]);
    }

    match parse_block(lines, 0, lines[0][0]) {
        Ok(pair) => {
            if pair[1] < lines.size() {
                return Err(
                    "yaml ligne " + str(lines[pair[1]][2]) + ": indentation inattendue"
                );
            }
            return Ok(pair[0]);
        }
        Err(message) => {
            return Err(message);
        }
    }
}

// ------------------------------------------------------------------
// Ecriture
// ------------------------------------------------------------------

func needs_quotes(text: str) -> bool {
    if text == "" || text != text.trim() {
        return true;
    }
    if text == "null" || text == "~" || text == "true" || text == "false"
        || text == "True" || text == "False" || text == "Null" {
        return true;
    }
    let first = text.substring(0, 1);
    if first == "-" || first == "[" || first == "{" || first == "&" || first == "*"
        || first == "!" || first == "|" || first == ">" || first == "'" || first == "\""
        || first == "#" || first == "%" || first == "@" {
        return true;
    }
    if text.contains(": ") || text.contains(" #") || text.contains("\n")
        || text.contains("\t") || text.ends_with(":") {
        return true;
    }
    // Ressemble a un nombre ?
    match parse_scalar(text) {
        Ok(value) => {
            return type(value) != "string";
        }
        Err(_) => {
            return true;
        }
    }
}

func encode_scalar(value) -> Result<str, str> {
    let kind = type(value);
    if kind == "None" {
        return Ok("null");
    }
    if kind == "bool" || kind == "int" || kind == "float" {
        return Ok(str(value));
    }
    if kind == "string" {
        if needs_quotes(value) {
            let escaped = value.replace_all("\\", "\\\\");
            escaped = escaped.replace_all("\"", "\\\"");
            escaped = escaped.replace_all("\n", "\\n");
            escaped = escaped.replace_all("\t", "\\t");
            return Ok("\"" + escaped + "\"");
        }
        return Ok(value);
    }
    return Err("yaml.stringify: type non representable (" + kind + ")");
}

func write_node(value, indent: int, output: List<str>) -> Result<bool, str> {
    let pad = " ".repeat(indent);
    let kind = type(value);

    if kind == "dict" {
        if value.is_empty() {
            output.add(pad + "{}");
            return Ok(true);
        }
        for key in value.keys() {
            let child = value.get(key);
            let child_kind = type(child);
            match encode_scalar(str(key)) {
                Ok(encoded_key) => {
                    if (child_kind == "dict" || child_kind == "list") && !child.is_empty() {
                        output.add(pad + encoded_key + ":");
                        match write_node(child, indent + 2, output) {
                            Ok(_) => {}
                            Err(message) => {
                                return Err(message);
                            }
                        }
                    } else if child_kind == "dict" {
                        output.add(pad + encoded_key + ": {}");
                    } else if child_kind == "list" {
                        output.add(pad + encoded_key + ": []");
                    } else {
                        match encode_scalar(child) {
                            Ok(encoded) => {
                                output.add(pad + encoded_key + ": " + encoded);
                            }
                            Err(message) => {
                                return Err(message);
                            }
                        }
                    }
                }
                Err(message) => {
                    return Err(message);
                }
            }
        }
        return Ok(true);
    }

    if kind == "list" {
        if value.is_empty() {
            output.add(pad + "[]");
            return Ok(true);
        }
        for item in value {
            let item_kind = type(item);
            if (item_kind == "dict" || item_kind == "list") && !item.is_empty() {
                // "-" seul, puis le bloc imbrique (toujours re-parsable).
                output.add(pad + "-");
                match write_node(item, indent + 2, output) {
                    Ok(_) => {}
                    Err(message) => {
                        return Err(message);
                    }
                }
            } else if item_kind == "dict" {
                output.add(pad + "- {}");
            } else if item_kind == "list" {
                output.add(pad + "- []");
            } else {
                match encode_scalar(item) {
                    Ok(encoded) => {
                        output.add(pad + "- " + encoded);
                    }
                    Err(message) => {
                        return Err(message);
                    }
                }
            }
        }
        return Ok(true);
    }

    match encode_scalar(value) {
        Ok(encoded) => {
            output.add(pad + encoded);
            return Ok(true);
        }
        Err(message) => {
            return Err(message);
        }
    }
}

export func stringify(value) -> Result<str, str> {
    let output = [];
    match write_node(value, 0, output) {
        Ok(_) => {
            return Ok(output.join("\n") + "\n");
        }
        Err(message) => {
            return Err(message);
        }
    }
}

export func read_file(path: str) -> Result<any, str> {
    try {
        return parse(file_read(path));
    } catch (error) {
        return Err(error);
    }
}

export func write_file(path: str, value) -> Result<bool, str> {
    match stringify(value) {
        Ok(text) => {
            try {
                file_write(path, text);
                return Ok(true);
            } catch (error) {
                return Err(error);
            }
        }
        Err(message) => {
            return Err(message);
        }
    }
}
