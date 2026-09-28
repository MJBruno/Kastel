// std/toml.ks
//
// Sous-ensemble de TOML en Kastel pur. SUPPORTÉ :
//   - commentaires `# ...` (hors chaînes) ;
//   - `clé = valeur` (clé nue ou entre guillemets) ;
//   - tables `[a]` et `[a.b]` (créées imbriquées) ;
//   - chaînes "basiques" (échappements \" \\ \n \t \r) et 'littérales' ;
//   - entiers (avec `_`), flottants, booléens ;
//   - tableaux sur UNE ligne, éventuellement imbriqués : [1, 2, [3]].
// NON SUPPORTÉ (renvoie Err plutôt que de mal interpréter) :
//   chaînes multi-lignes ("""), tables inline {..}, tableaux de tables
//   [[x]], clés pointées dans une ligne (a.b = 1), dates/heures,
//   tableaux étalés sur plusieurs lignes.
//
// Le résultat est un Dict : tables -> Dict imbriqués, tableaux -> List.
//
// Usage :
//
//   import std.toml;
//   match toml.parse("titre = \"x\"\n[db]\nport = 5432\n") {
//       Ok(config) => { println(config.get("db").get("port")); }
//       Err(message) => { println(message); }
//   }

// Retire un commentaire de fin de ligne, en ignorant les `#` dans une
// chaîne.
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
        } else if ch == "#" {
            return line.substring(0, i);
        }
        i = i + 1;
    }
    return line;
}

// Decoupe `text` sur `,` au niveau racine (hors chaines et hors [...]).
func split_top_level(text: str) -> List<str> {
    let parts = [];
    let current = "";
    let depth = 0;
    let quote = "";
    let i = 0;
    while i < text.size() {
        let ch = text.substring(i, 1);
        if quote != "" {
            current = current + ch;
            if ch == "\\" && quote == "\"" && i + 1 < text.size() {
                i = i + 1;
                current = current + text.substring(i, 1);
            } else if ch == quote {
                quote = "";
            }
        } else if ch == "\"" || ch == "'" {
            quote = ch;
            current = current + ch;
        } else if ch == "[" {
            depth = depth + 1;
            current = current + ch;
        } else if ch == "]" {
            depth = depth - 1;
            current = current + ch;
        } else if ch == "," && depth == 0 {
            parts.add(current);
            current = "";
        } else {
            current = current + ch;
        }
        i = i + 1;
    }
    if current.trim().size() > 0 {
        parts.add(current);
    }
    return parts;
}

func parse_basic_string(text: str) -> Result<str, str> {
    // `text` = "..." complet (guillemets inclus).
    let inner = text.substring(1, text.size() - 2);
    let result = "";
    let i = 0;
    while i < inner.size() {
        let ch = inner.substring(i, 1);
        if ch == "\\" {
            if i + 1 >= inner.size() {
                return Err("toml: echappement incomplet dans " + text);
            }
            i = i + 1;
            let next = inner.substring(i, 1);
            if next == "n" {
                result = result + "\n";
            } else if next == "t" {
                result = result + "\t";
            } else if next == "r" {
                result = result + "\r";
            } else if next == "\"" || next == "\\" {
                result = result + next;
            } else {
                return Err("toml: echappement non supporte \\" + next + " dans " + text);
            }
        } else {
            result = result + ch;
        }
        i = i + 1;
    }
    return Ok(result);
}

func parse_value(raw: str) -> Result<any, str> {
    let text = raw.trim();
    if text.size() == 0 {
        return Err("toml: valeur manquante");
    }

    if text == "true" {
        return Ok(true);
    }
    if text == "false" {
        return Ok(false);
    }

    let first = text.substring(0, 1);

    if text.starts_with("\"\"\"") || text.starts_with("'''") {
        return Err("toml: chaines multi-lignes non supportees");
    }

    if first == "\"" {
        if text.size() < 2 || text.substring(text.size() - 1, 1) != "\"" {
            return Err("toml: chaine non terminee: " + text);
        }
        return parse_basic_string(text);
    }

    if first == "'" {
        if text.size() < 2 || text.substring(text.size() - 1, 1) != "'" {
            return Err("toml: chaine non terminee: " + text);
        }
        return Ok(text.substring(1, text.size() - 2));
    }

    if first == "[" {
        if text.substring(text.size() - 1, 1) != "]" {
            return Err("toml: tableau non termine (les tableaux multi-lignes ne sont pas supportes): " + text);
        }
        let items = [];
        let inner = text.substring(1, text.size() - 2);
        for part in split_top_level(inner) {
            match parse_value(part) {
                Ok(item) => {
                    items.add(item);
                }
                Err(message) => {
                    return Err(message);
                }
            }
        }
        return Ok(items);
    }

    if first == "{" {
        return Err("toml: tables inline non supportees");
    }

    let number = text.replace_all("_", "");
    try {
        if number.contains(".") || number.contains("e") || number.contains("E") {
            return Ok(number.to_float());
        }
        return Ok(number.to_int());
    } catch (_) {
        return Err("toml: valeur non reconnue: " + text);
    }
}

func parse_key(raw: str) -> Result<str, str> {
    let key = raw.trim();
    if key.size() == 0 {
        return Err("toml: cle vide");
    }
    let first = key.substring(0, 1);
    if first == "\"" {
        if key.size() < 2 || key.substring(key.size() - 1, 1) != "\"" {
            return Err("toml: cle non terminee: " + key);
        }
        return parse_basic_string(key);
    }
    if first == "'" {
        if key.size() < 2 || key.substring(key.size() - 1, 1) != "'" {
            return Err("toml: cle non terminee: " + key);
        }
        return Ok(key.substring(1, key.size() - 2));
    }
    if key.contains(".") {
        return Err("toml: cles pointees non supportees dans une ligne: " + key);
    }
    let i = 0;
    while i < key.size() {
        let ch = key.substring(i, 1);
        if !(ch.is_alphanumeric() || ch == "_" || ch == "-") {
            return Err("toml: caractere invalide dans la cle: " + key);
        }
        i = i + 1;
    }
    return Ok(key);
}

// Descend dans `root` en creant les tables manquantes ; renvoie la
// table finale.
func ensure_table(root, path: List<str>) -> Result<any, str> {
    let current = root;
    for name in path {
        if current.contains(name) {
            let existing = current.get(name);
            if type(existing) != "dict" {
                return Err("toml: '" + name + "' est deja defini et n'est pas une table");
            }
            current = existing;
        } else {
            let created = {};
            current.set(name, created);
            current = created;
        }
    }
    return Ok(current);
}

export func parse(text: str) -> Result<Dict<str, any>, str> {
    let root = {};
    let current = root;
    let line_number = 0;

    for raw_line in text.split("\n") {
        line_number = line_number + 1;
        let line = strip_comment(raw_line).trim();
        if line.size() == 0 {
            continue;
        }

        if line.starts_with("[[") {
            return Err("toml ligne " + str(line_number) + ": tableaux de tables [[..]] non supportes");
        }

        if line.starts_with("[") {
            if !line.ends_with("]") {
                return Err("toml ligne " + str(line_number) + ": en-tete de table non termine");
            }
            let names = [];
            for part in line.substring(1, line.size() - 2).split(".") {
                match parse_key(part) {
                    Ok(name) => {
                        names.add(name);
                    }
                    Err(message) => {
                        return Err("toml ligne " + str(line_number) + ": " + message);
                    }
                }
            }
            match ensure_table(root, names) {
                Ok(table) => {
                    current = table;
                }
                Err(message) => {
                    return Err("toml ligne " + str(line_number) + ": " + message);
                }
            }
            continue;
        }

        let equals_at = line.index_of("=");
        if equals_at < 0 {
            return Err("toml ligne " + str(line_number) + ": '=' attendu");
        }

        match parse_key(line.substring(0, equals_at)) {
            Ok(key) => {
                if current.contains(key) {
                    return Err("toml ligne " + str(line_number) + ": cle en double '" + key + "'");
                }
                match parse_value(line.substring(equals_at + 1, line.size() - equals_at - 1)) {
                    Ok(value) => {
                        current.set(key, value);
                    }
                    Err(message) => {
                        return Err("toml ligne " + str(line_number) + ": " + message);
                    }
                }
            }
            Err(message) => {
                return Err("toml ligne " + str(line_number) + ": " + message);
            }
        }
    }

    return Ok(root);
}

// ------------------------------------------------------------------
// Ecriture
// ------------------------------------------------------------------

func encode_string(text: str) -> str {
    let escaped = text.replace_all("\\", "\\\\");
    escaped = escaped.replace_all("\"", "\\\"");
    escaped = escaped.replace_all("\n", "\\n");
    escaped = escaped.replace_all("\t", "\\t");
    escaped = escaped.replace_all("\r", "\\r");
    return "\"" + escaped + "\"";
}

func encode_value(value) -> Result<str, str> {
    let kind = type(value);
    if kind == "string" {
        return Ok(encode_string(value));
    }
    if kind == "int" || kind == "float" || kind == "bool" {
        return Ok(str(value));
    }
    if kind == "list" {
        let parts = [];
        for item in value {
            match encode_value(item) {
                Ok(encoded) => {
                    parts.add(encoded);
                }
                Err(message) => {
                    return Err(message);
                }
            }
        }
        return Ok("[" + parts.join(", ") + "]");
    }
    return Err("toml.stringify: type non representable (" + kind + ")");
}

func write_table(table, prefix: str, output: List<str>) -> Result<bool, str> {
    // 1) valeurs simples de cette table, 2) sous-tables ensuite (TOML
    // exige que les valeurs precedent les en-tetes de sous-tables).
    let sub_tables = [];
    for key in table.keys() {
        let value = table.get(key);
        if type(value) == "dict" {
            sub_tables.add(key);
        } else {
            match encode_value(value) {
                Ok(encoded) => {
                    output.add(key + " = " + encoded);
                }
                Err(message) => {
                    return Err(message + " pour la cle '" + key + "'");
                }
            }
        }
    }

    for key in sub_tables {
        let path = key;
        if prefix != "" {
            path = prefix + "." + key;
        }
        output.add("");
        output.add("[" + path + "]");
        match write_table(table.get(key), path, output) {
            Ok(_) => {}
            Err(message) => {
                return Err(message);
            }
        }
    }
    return Ok(true);
}

// Limite : les cles sont ecrites telles quelles (pas de mise entre
// guillemets) ; elles doivent donc etre des identifiants TOML "nus"
// (lettres, chiffres, _ et -), sinon la sortie ne serait pas du TOML
// valide.
export func stringify(data: Dict<str, any>) -> Result<str, str> {
    let lines = [];
    match write_table(data, "", lines) {
        Ok(_) => {
            return Ok(lines.join("\n") + "\n");
        }
        Err(message) => {
            return Err(message);
        }
    }
}

export func read_file(path: str) -> Result<Dict<str, any>, str> {
    try {
        return parse(file_read(path));
    } catch (error) {
        return Err(error);
    }
}

export func write_file(path: str, data: Dict<str, any>) -> Result<bool, str> {
    match stringify(data) {
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
