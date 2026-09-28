// std/core.ks
//
// Petit "prélude" transversal : prédicats de type au-dessus de type()
// natif (system.rs), garde-fous génériques (require), et quelques
// combinateurs fonctionnels de base. Rien ici n'est spécifique à un
// domaine (fichiers, dates, réseau, ...) — c'est ce que les autres
// modules std.* utilisent en commun.

// ------------------------------------------------------------------
// Prédicats de type (type() natif renvoie une chaîne : "int", "float",
// "bool", "string", "list", "dict", "set", "tuple", "Option", "Result",
// "None", "function", ou le nom de la classe pour une instance).
// ------------------------------------------------------------------

export func is_int(value) -> bool {
    return type(value) == "int";
}

export func is_float(value) -> bool {
    return type(value) == "float";
}

// int ET float : pratique pour valider "un nombre" sans se soucier de
// la représentation exacte.
export func is_number(value) -> bool {
    let kind = type(value);
    return kind == "int" || kind == "float";
}

export func is_bool(value) -> bool {
    return type(value) == "bool";
}

export func is_string(value) -> bool {
    return type(value) == "string";
}

export func is_list(value) -> bool {
    return type(value) == "list";
}

export func is_dict(value) -> bool {
    return type(value) == "dict";
}

export func is_set(value) -> bool {
    return type(value) == "set";
}

export func is_tuple(value) -> bool {
    return type(value) == "tuple";
}

export func is_none(value) -> bool {
    return type(value) == "None";
}

export func is_function(value) -> bool {
    return type(value) == "function";
}

// Nom exact du type/de la classe (raccourci sur type() natif, pour ne
// pas avoir à l'importer séparément quand on importe déjà std.core).
export func type_name(value) -> str {
    return type(value);
}

// ------------------------------------------------------------------
// Garde-fous
// ------------------------------------------------------------------

// Comme un `assert` de langage systeme : leve `message` si `condition`
// est fausse. Contrairement a std.testing (pense pour des tests),
// `require` est destine au code applicatif (valider un argument, un
// invariant).
export func require(condition: bool, message: str) {
    if !condition {
        throw message;
    }
}

// Arrete le programme immediatement avec `message`. Simple alias de
// `throw` sous forme de fonction, pratique en bout de `match`
// (`_ => { panic("cas impossible"); }`) ou le corps doit rester une
// expression/instruction plutot qu'un `throw` nu.
export func panic(message: str) {
    throw message;
}

// ------------------------------------------------------------------
// Combinateurs fonctionnels generiques
// ------------------------------------------------------------------

export func identity<T>(value: T) -> T {
    return value;
}

// Renvoie une fonction qui ignore son argument et renvoie toujours
// `value` — pratique comme callback par defaut (ex. Dict.update()).
export func constant<T>(value: T) {
    return func(ignored) {
        return value;
    };
}

// compose(f, g)(x) == f(g(x)). Les deux callbacks restent dynamiques
// (pas de contrainte de type imposable sur une fonction generique ici).
export func compose(f, g) {
    return func(value) {
        return f(g(value));
    };
}

// pipe(x, [f, g, h]) == h(g(f(x))) : applique une liste de fonctions
// dans l'ordre de lecture, souvent plus lisible que compose() imbrique
// pour une chaine de transformations.
export func pipe(value, steps) {
    let result = value;
    for step in steps {
        result = step(result);
    }
    return result;
}

// Repete `action` (fonction sans argument) `times` fois de suite.
export func repeat(times: int, action) {
    let i = 0;
    while i < times {
        action();
        i = i + 1;
    }
}
