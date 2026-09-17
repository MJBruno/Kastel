//! Constantes du langage Kastel partagées entre le LSP et l'analyse.
//!
//! Ces listes doivent rester synchronisées avec le lexer/parser/stdlib
//! réels du crate `kastel` (dépendance de chemin `..`). En cas de
//! doute sur un nom, se référer à :
//!   - `src/frontend/lexer/token.rs` (`Token::keyword`) pour les mots-clés
//!   - `src/stdlib/*.rs` (`register` / `dispatch_method`) pour la stdlib

/// Mots-clés réservés du langage.
pub const KEYWORDS: &[&str] = &[
    "const", "let", "func", "return", "if", "else", "while", "for", "in", "match", "break",
    "continue", "import", "from", "as", "export", "class", "new", "this", "base", "interface",
    "try", "catch", "throw", "finally", "is", "true", "false", "None",
];

/// Fonctions/valeurs globales fournies par le runtime Kastel :
/// `(nom, signature affichée, courte description)`.
///
/// À étendre au fur et à mesure que la stdlib grossit — voir
/// `src/stdlib/mod.rs::register_natives` dans le crate `kastel` pour
/// la liste faisant foi.
pub const BUILTIN_FUNCTIONS: &[(&str, &str, &str)] = &[
    // -- io --
    ("print", "print(value)", "Affiche une valeur sans retour à la ligne."),
    ("println", "println(value)", "Affiche une valeur suivie d'un retour à la ligne."),
    ("input", "input(prompt?)", "Lit une ligne depuis l'entrée standard."),
    // -- math --
    ("rand", "rand()", "Nombre flottant aléatoire entre 0.0 et 1.0."),
    ("rand_int", "rand_int(min, max)", "Entier aléatoire dans [min, max]."),
    ("rand_range", "rand_range(start, end)", "Entier aléatoire dans [start, end)."),
    ("abs", "abs(x)", "Valeur absolue."),
    ("floor", "floor(x)", "Arrondi à l'entier inférieur (renvoie un entier)."),
    ("ceil", "ceil(x)", "Arrondi à l'entier supérieur (renvoie un entier)."),
    ("round", "round(x)", "Arrondi au plus proche, pair en cas d'égalité (renvoie un entier)."),
    ("sqrt", "sqrt(x)", "Racine carrée (renvoie toujours un flottant)."),
    ("pow", "pow(base, exp)", "Puissance. Entier si base et exposant (≥0) sont entiers, flottant sinon."),
    ("min", "min(a, b)", "Le plus petit des deux."),
    ("max", "max(a, b)", "Le plus grand des deux."),
    ("sin", "sin(x)", "Sinus (radians)."),
    ("cos", "cos(x)", "Cosinus (radians)."),
    ("tan", "tan(x)", "Tangente (radians)."),
    ("log", "log(x)", "Logarithme naturel."),
    ("log10", "log10(x)", "Logarithme base 10."),
    ("exp", "exp(x)", "Exponentielle."),
    // -- string --
    ("format", "format(template, ...args)", "Interpole `{}` dans `template` avec les arguments."),
    // -- iterator --
    ("range", "range(stop) / range(start, stop) / range(start, stop, step)", "Séquence d'entiers, utilisable dans `for x in range(...)`."),
    ("list", "list(iterable)", "Convertit un itérable en tableau."),
    // -- dict --
    ("dict", "dict()", "Crée un dictionnaire vide."),
    // -- system --
    ("int", "int(value)", "Convertit vers un entier."),
    ("float", "float(value)", "Convertit vers un flottant."),
    ("str", "str(value)", "Convertit vers une chaîne."),
    ("bool", "bool(value)", "Convertit vers un booléen."),
    ("type", "type(value)", "Nom du type de la valeur (\"integer\", \"string\", ...)."),
    ("clock", "clock()", "Horodatage courant."),
    ("cwd", "cwd()", "Répertoire de travail courant."),
    ("env", "env(name)", "Lit une variable d'environnement."),
    // -- debug --
    ("inspect", "inspect(value)", "Représentation détaillée d'une valeur (stderr)."),
    ("debug", "debug(...)", "Trace de debug (stderr)."),
];

/// Juste les noms, pour les vérifications rapides (ex. filtrage des
/// faux positifs « identifiant non défini » dans `semantic.rs`).
pub const BUILTINS: &[&str] = &[
    "print", "println", "input", "rand", "rand_int", "rand_range", "abs", "floor", "ceil",
    "round", "sqrt", "pow", "min", "max", "sin", "cos", "tan", "log", "log10", "exp", "format",
    "range", "list", "dict", "int", "float", "str", "bool", "type", "clock", "cwd", "env",
    "inspect", "debug",
];

/// Méthodes disponibles sur un tableau (`array`) : `(nom, signature, doc)`.
///
/// NOTE : `length` est une **propriété** (`arr.length`, sans
/// parenthèses) et non une méthode — `arr.length()` échoue à
/// l'exécution. Elle est listée ici avec une signature sans
/// parenthèses pour refléter cela dans la complétion/hover.
pub const ARRAY_METHODS: &[(&str, &str, &str)] = &[
    ("length", "arr.length", "Nombre d'éléments (propriété, sans parenthèses)."),
    ("push", "arr.push(value)", "Ajoute un élément à la fin."),
    ("pop", "arr.pop()", "Retire et renvoie le dernier élément."),
    ("insert", "arr.insert(index, value)", "Insère `value` à `index`."),
    ("remove", "arr.remove(index)", "Retire et renvoie l'élément à `index`."),
    ("get", "arr.get(index)", "Élément à `index`."),
    ("set", "arr.set(index, value)", "Remplace l'élément à `index`."),
    ("contains", "arr.contains(value)", "Vrai si `value` est présente."),
    ("index_of", "arr.index_of(value)", "Index de la première occurrence de `value`."),
    ("slice", "arr.slice(start, end)", "Sous-tableau `[start, end)`."),
    ("reverse", "arr.reverse()", "Inverse le tableau en place."),
    ("join", "arr.join(separator)", "Concatène les éléments en une chaîne."),
    ("clear", "arr.clear()", "Vide le tableau."),
    ("first", "arr.first()", "Premier élément."),
    ("last", "arr.last()", "Dernier élément."),
    ("sort", "arr.sort()", "Trie le tableau en place, par ordre croissant."),
    ("copy", "arr.copy()", "Copie superficielle du tableau."),
    ("map", "arr.map(fn)", "Nouveau tableau via `fn` appliquée à chaque élément."),
    ("filter", "arr.filter(fn)", "Nouveau tableau des éléments où `fn` est vrai."),
    ("reduce", "arr.reduce(fn, initial)", "Réduit le tableau à une seule valeur."),
    ("any", "arr.any(fn)", "Vrai si `fn` est vrai pour au moins un élément."),
    ("all", "arr.all(fn)", "Vrai si `fn` est vrai pour tous les éléments."),
];

/// Méthodes disponibles sur une chaîne (`string`).
pub const STRING_METHODS: &[(&str, &str, &str)] = &[
    ("length", "s.length()", "Nombre de caractères."),
    ("upper", "s.upper()", "Version en majuscules."),
    ("lower", "s.lower()", "Version en minuscules."),
    ("contains", "s.contains(needle)", "Vrai si `needle` apparaît dans la chaîne."),
    ("starts_with", "s.starts_with(prefix)", "Vrai si la chaîne commence par `prefix`."),
    ("ends_with", "s.ends_with(suffix)", "Vrai si la chaîne se termine par `suffix`."),
    ("index_of", "s.index_of(needle)", "Index de la première occurrence de `needle`."),
    ("last_index_of", "s.last_index_of(needle)", "Index de la dernière occurrence de `needle`."),
    ("replace", "s.replace(from, to)", "Remplace la première occurrence de `from` par `to`."),
    ("replace_all", "s.replace_all(from, to)", "Remplace toutes les occurrences de `from` par `to`."),
    ("split", "s.split(separator)", "Découpe la chaîne en tableau."),
    ("substring", "s.substring(start, length)", "Sous-chaîne de `length` caractères à partir de `start`."),
    ("slice", "s.slice(start, end)", "Sous-chaîne `[start, end)`."),
    ("trim", "s.trim()", "Retire les espaces en début et fin."),
    ("trim_start", "s.trim_start()", "Retire les espaces en début."),
    ("trim_end", "s.trim_end()", "Retire les espaces en fin."),
    ("repeat", "s.repeat(n)", "Répète la chaîne `n` fois."),
    ("reverse", "s.reverse()", "Chaîne inversée."),
    ("char_at", "s.char_at(index)", "Caractère à `index`."),
    ("get", "s.get(index)", "Caractère à `index`."),
    ("join", "s.join(array)", "Utilise la chaîne comme séparateur pour joindre `array`."),
    ("is_empty", "s.is_empty()", "Vrai si la chaîne est vide."),
    ("is_digit", "s.is_digit()", "Vrai si tous les caractères sont des chiffres."),
    ("is_alpha", "s.is_alpha()", "Vrai si tous les caractères sont alphabétiques."),
    ("is_alphanumeric", "s.is_alphanumeric()", "Vrai si tous les caractères sont alphanumériques."),
    ("to_int", "s.to_int()", "Conversion en entier."),
    ("to_float", "s.to_float()", "Conversion en flottant."),
];

/// Méthodes disponibles sur un dictionnaire (`dict`).
pub const DICT_METHODS: &[(&str, &str, &str)] = &[
    ("length", "d.length()", "Nombre de paires clé/valeur."),
    ("get", "d.get(key)", "Valeur associée à `key`."),
    ("get_or", "d.get_or(key, default)", "Valeur associée à `key`, ou `default` si absente."),
    ("set", "d.set(key, value)", "Associe `value` à `key`."),
    ("has", "d.has(key)", "Vrai si `key` est présente."),
    ("remove", "d.remove(key)", "Retire `key` et renvoie sa valeur."),
    ("keys", "d.keys()", "Tableau des clés, dans l'ordre d'insertion."),
    ("values", "d.values()", "Tableau des valeurs, dans l'ordre d'insertion."),
    ("items", "d.items()", "Tableau de paires [clé, valeur]."),
    ("clear", "d.clear()", "Vide le dictionnaire."),
    ("copy", "d.copy()", "Copie superficielle du dictionnaire."),
    ("update", "d.update(other)", "Fusionne les paires de `other`."),
];

/// Méthodes disponibles sur un tuple (`tuple`) — immuable.
pub const TUPLE_METHODS: &[(&str, &str, &str)] = &[
    ("length", "t.length", "Nombre d'éléments (propriété OU méthode : `t.length` et `t.length()` marchent tous les deux)."),
    ("get", "t.get(index)", "Élément à `index`."),
    ("contains", "t.contains(value)", "Vrai si `value` est présente."),
    ("index_of", "t.index_of(value)", "Index de la première occurrence de `value`."),
    ("first", "t.first()", "Premier élément."),
    ("last", "t.last()", "Dernier élément."),
    ("to_array", "t.to_array()", "Copie les éléments dans un nouveau tableau (mutable)."),
];
