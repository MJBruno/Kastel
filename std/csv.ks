// std/csv.ks
//
// CSV en Kastel pur, machine à états caractère par caractère (RFC 4180) :
//   - champs entre guillemets, `""` = guillemet littéral ;
//   - séparateur et retours à la ligne autorisés DANS un champ entre
//     guillemets ;
//   - fins de ligne \n ou \r\n acceptées.
// Une guillemet non refermée est une erreur (Err), jamais silencieusement
// ignorée.
//
// Usage :
//
//   import std.csv;
//   match csv.parse_with_header("nom,age\nAda,36\n", ",") {
//       Ok(rows) => { println(rows[0].get("nom")); }
//       Err(message) => { println(message); }
//   }

// Découpe `text` en List<List<str>>. `delimiter` : un seul caractère.
export func parse(text: str, delimiter: str) -> Result<List<List<str>>, str> {
    if delimiter.size() != 1 {
        return Err("csv.parse: le separateur doit faire exactement 1 caractere");
    }
    if delimiter == "\"" || delimiter == "\n" || delimiter == "\r" {
        return Err("csv.parse: separateur invalide");
    }

    let rows = [];
    let row = [];
    let field = "";
    let in_quotes = false;
    // Vrai si la ligne courante a deja recu un champ ou un separateur :
    // distingue une vraie ligne vide (ignoree) d'une ligne "a,".
    let row_started = false;

    let i = 0;
    let count = text.size();
    while i < count {
        let ch = text.substring(i, 1);

        if in_quotes {
            if ch == "\"" {
                if i + 1 < count && text.substring(i + 1, 1) == "\"" {
                    field = field + "\"";
                    i = i + 1;
                } else {
                    in_quotes = false;
                }
            } else {
                field = field + ch;
            }
        } else if ch == "\"" {
            if field.size() > 0 {
                return Err("csv.parse: guillemet inattendue au milieu d'un champ (caractere " + str(i) + ")");
            }
            in_quotes = true;
            row_started = true;
        } else if ch == delimiter {
            row.add(field);
            field = "";
            row_started = true;
        } else if ch == "\n" || ch == "\r" {
            // "\r\n" : on consomme le "\n" qui suit le "\r".
            if ch == "\r" && i + 1 < count && text.substring(i + 1, 1) == "\n" {
                i = i + 1;
            }
            if row_started || field.size() > 0 {
                row.add(field);
                rows.add(row);
            }
            row = [];
            field = "";
            row_started = false;
        } else {
            field = field + ch;
            row_started = true;
        }

        i = i + 1;
    }

    if in_quotes {
        return Err("csv.parse: guillemet ouvrante jamais refermee");
    }

    // Derniere ligne sans retour a la ligne final.
    if row_started || field.size() > 0 {
        row.add(field);
        rows.add(row);
    }

    return Ok(rows);
}

export func parse_default(text: str) -> Result<List<List<str>>, str> {
    return parse(text, ",");
}

// Premiere ligne = en-tetes ; renvoie une List<Dict<str, str>>, une
// entree par ligne de donnees. Err si une ligne n'a pas autant de
// colonnes que l'en-tete (le numero de ligne du fichier est indique).
export func parse_with_header(text: str, delimiter: str) -> Result<List<Dict<str, str>>, str> {
    match parse(text, delimiter) {
        Ok(rows) => {
            if rows.is_empty() {
                return Ok([]);
            }

            let header = rows[0];
            let records = [];
            let line = 1;
            while line < rows.size() {
                let values = rows[line];
                if values.size() != header.size() {
                    return Err(
                        "csv.parse_with_header: ligne " + str(line + 1) + " a "
                            + str(values.size()) + " colonne(s), l'en-tete en a "
                            + str(header.size())
                    );
                }

                let record = {};
                let column = 0;
                while column < header.size() {
                    record.set(header[column], values[column]);
                    column = column + 1;
                }
                records.add(record);
                line = line + 1;
            }
            return Ok(records);
        }
        Err(message) => {
            return Err(message);
        }
    }
}

func needs_quoting(field: str, delimiter: str) -> bool {
    return field.contains(delimiter)
        || field.contains("\"")
        || field.contains("\n")
        || field.contains("\r");
}

func encode_field(field: str, delimiter: str) -> str {
    if needs_quoting(field, delimiter) {
        return "\"" + field.replace_all("\"", "\"\"") + "\"";
    }
    return field;
}

// Ecrit `rows` en CSV (fin de ligne \n, y compris apres la derniere
// ligne). Les valeurs non-str sont converties avec str().
export func stringify(rows: List<List<any>>, delimiter: str) -> str {
    let output = "";
    for row in rows {
        let fields = [];
        for cell in row {
            fields.add(encode_field(str(cell), delimiter));
        }
        output = output + fields.join(delimiter) + "\n";
    }
    return output;
}

export func read_file(path: str, delimiter: str) -> Result<List<List<str>>, str> {
    try {
        return parse(file_read(path), delimiter);
    } catch (error) {
        return Err(error);
    }
}

export func write_file(path: str, rows: List<List<any>>, delimiter: str) -> Result<bool, str> {
    try {
        file_write(path, stringify(rows, delimiter));
        return Ok(true);
    } catch (error) {
        return Err(error);
    }
}
