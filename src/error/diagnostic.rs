// ================================================================
// DIAGNOSTIC (rendu façon rustc)
// ================================================================
//
// Ce module ne connaît rien des erreurs Kastel elles-mêmes : il reçoit un
// `Diagnostic` déjà construit (titre, position, éventuels expected/found/
// help) ainsi que le texte source complet, et produit un encart du genre :
//
//   error: cannot add a string and an integer
//
//     --> main.ks:8:15
//      |
//    8 | println(name + age)
//      |               ^^^
//      |
//      = expected: string
//      = found: integer
//
//   help: convert the value to a string:
//         println(name + str(age))
//
// La précision de la position (ligne/colonne) dépend de ce que la phase
// d'origine (lexer, parser, compilateur, VM) a effectivement suivi — voir
// les commentaires dans compile_error.rs / runtime_error.rs. Quand seule
// une position "au niveau de l'instruction" est disponible (pas au niveau
// exact de l'opérande fautif), le caret pointe sur le début de la ligne
// plutôt que d'inventer une colonne précise.

#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Résumé de l'erreur, affiché après `error: ` (en une ligne, sans
    /// majuscule ni point final, dans l'esprit rustc).
    pub title: String,

    /// Ligne 1-indexée dans le fichier source.
    pub line: usize,

    /// Colonne 1-indexée dans la ligne.
    pub column: usize,

    /// Largeur du soulignement (`^^^`), minimum 1.
    pub len: usize,

    pub expected: Option<String>,
    pub found: Option<String>,

    /// Suggestion actionnable, éventuellement multi-lignes (la première
    /// ligne suit `help: ` directement, les suivantes sont indentées pour
    /// s'aligner dessous).
    pub help: Option<String>,
}

impl Diagnostic {
    pub fn new(title: impl Into<String>, line: usize, column: usize) -> Self {
        Self {
            title: title.into(),
            line,
            column,
            len: 1,
            expected: None,
            found: None,
            help: None,
        }
    }

    pub fn with_len(mut self, len: usize) -> Self {
        self.len = len.max(1);
        self
    }

    pub fn with_expected(mut self, expected: impl Into<String>) -> Self {
        self.expected = Some(expected.into());
        self
    }

    pub fn with_found(mut self, found: impl Into<String>) -> Self {
        self.found = Some(found.into());
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Largeur effective du soulignement. Quand `len` n'a pas été fixée
    /// explicitement (valeur par défaut 1), on essaie de deviner la
    /// largeur du token source à `column` en élargissant sur les
    /// caractères alphanumériques/`_` contigus (identifiants, nombres) —
    /// ça évite de devoir faire remonter une longueur exacte depuis
    /// chaque site d'erreur pour obtenir un caret qui couvre bien
    /// l'opérande visé plutôt qu'un unique `^`. Une `len` fixée
    /// explicitement (> 1, ou volontairement à 1) est toujours respectée
    /// telle quelle.
    fn effective_len(&self, source_line: &str) -> usize {
        if self.len != 1 {
            return self.len;
        }

        let chars: Vec<char> = source_line.chars().collect();
        let start = self.column.saturating_sub(1);

        let is_word = |c: char| c.is_alphanumeric() || c == '_';

        if start >= chars.len() || !is_word(chars[start]) {
            return self.len;
        }

        let mut end = start;

        while end < chars.len() && is_word(chars[end]) {
            end += 1;
        }

        end - start
    }

    /// Rend le diagnostic complet avec l'extrait de code source, dans le
    /// style "rustc" décrit en en-tête de fichier. `file` est le nom
    /// affiché après `-->` (chemin du fichier, ou un nom conventionnel
    /// comme `<repl>`).
    pub fn render(&self, source: &str, file: &str) -> String {
        let lines: Vec<&str> = source.lines().collect();

        let line_number = self.line.max(1);
        let source_line = lines.get(line_number - 1).copied().unwrap_or("");

        // Largeur de la colonne "numéro de ligne" : chiffres du numéro,
        // encadrés d'un espace de chaque côté (ex. " 8 " -> largeur 3).
        let gutter_width = line_number.to_string().len() + 2;
        let empty_gutter = " ".repeat(gutter_width);
        // La ligne "--> fichier:ligne:colonne" est alignée un cran plus à
        // gauche que les lignes "|", comme dans l'exemple de référence.
        let arrow_indent = " ".repeat(gutter_width.saturating_sub(1));

        let mut output = String::new();

        output.push_str(&format!("error: {}\n\n", self.title));
        output.push_str(&format!(
            "{arrow_indent}--> {file}:{}:{}\n",
            self.line, self.column
        ));
        output.push_str(&format!("{empty_gutter}|\n"));
        output.push_str(&format!(
            " {:>width$} | {source_line}\n",
            line_number,
            width = gutter_width.saturating_sub(2)
        ));

        let caret_padding = " ".repeat(self.column.saturating_sub(1));
        let caret_len = self.effective_len(source_line);
        let caret = "^".repeat(caret_len.max(1));

        output.push_str(&format!("{empty_gutter}| {caret_padding}{caret}\n"));

        if self.expected.is_some() || self.found.is_some() {
            output.push_str(&format!("{empty_gutter}|\n"));

            if let Some(expected) = &self.expected {
                output.push_str(&format!("{empty_gutter}= expected: {expected}\n"));
            }

            if let Some(found) = &self.found {
                output.push_str(&format!("{empty_gutter}= found: {found}\n"));
            }
        }

        if let Some(help) = &self.help {
            output.push('\n');

            for (index, line) in help.lines().enumerate() {
                if index == 0 {
                    output.push_str("help: ");
                } else {
                    output.push_str("      ");
                }

                output.push_str(line);
                output.push('\n');
            }
        }

        // Retire le dernier saut de ligne : le code appelant gère
        // lui-même l'espacement entre plusieurs diagnostics.
        if output.ends_with('\n') {
            output.pop();
        }

        output
    }
}
