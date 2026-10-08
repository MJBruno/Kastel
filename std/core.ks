// std/core.ks
// Prélude transversal officiel de Kastel.

export func is_int(value: dynamic) -> bool {
    return type(value) == "int";
}

export func is_float(value: dynamic) -> bool {
    return type(value) == "float";
}

export func is_number(value: dynamic) -> bool {
    let kind = type(value);
    return kind == "int" || kind == "float";
}

export func is_bool(value: dynamic) -> bool {
    return type(value) == "bool";
}

export func is_string(value: dynamic) -> bool {
    return type(value) == "string";
}

export func is_list(value: dynamic) -> bool {
    return type(value) == "list";
}

export func is_dict(value: dynamic) -> bool {
    return type(value) == "dict";
}

export func is_set(value: dynamic) -> bool {
    return type(value) == "set";
}

export func is_tuple(value: dynamic) -> bool {
    return type(value) == "tuple";
}

export func is_none(value: dynamic) -> bool {
    return type(value) == "None";
}

export func is_function(value: dynamic) -> bool {
    return type(value) == "function";
}

export func type_name(value: dynamic) -> str {
    return type(value);
}

// Garde-fou applicatif : lève une exception lorsque l'invariant est faux.
export func require(condition: bool, message: str) -> None {
    if !condition {
        throw message;
    }
}

export func panic(message: str) -> None {
    throw message;
}

export func identity<T>(value: T) -> T {
    return value;
}

export func constant<T>(value: T) -> func() -> T {
    return func() {
        return value;
    };
}

export func compose<A, B, C>(
    f: func(B) -> C,
    g: func(A) -> B
) -> func(A) -> C {
    return func(value) {
        return f(g(value));
    };
}

export func pipe(value: dynamic, steps: List<dynamic>) -> dynamic {
    let result = value;
    for step in steps {
        result = step(result);
    }
    return result;
}

export func repeat(times: int, action: func() -> None) -> None {
    if times < 0 {
        throw "repeat: times doit etre positif";
    }
    let i = 0;
    while i < times {
        action();
        i = i + 1;
    }
}
