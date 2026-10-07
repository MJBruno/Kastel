# std.collections

Combinateurs de collections et d'itérateurs complémentaires aux méthodes natives de `List`, `Set`, `Dict` et `Tuple`.

## Conventions

La surface officielle utilise `List`, `size()`, `is_empty()`, `contains()`, `add()`, `remove()` et `copy()` selon la collection concernée.

## Principaux utilitaires

`find`, `find_index`, `partition`, `flatten_options`, `try_get`, `try_remove`, `zip`, `flat_map`, `chunks`, `group_by`, `count_by` et les variantes `_opt`/`try_map`.

```Rust
import std.collections;

let values = [10, 20, 30];

match collections.first_opt(values) {
    Some(value) => println(value);
    None => println("liste vide");
}

match collections.get_opt(values, 1) {
    Some(value) => println(value);
    None => println("index invalide");
}

let pairs = collections.zip(["a", "b"], [1, 2]);
println(pairs);

let chunks = collections.chunks([1, 2, 3, 4, 5], 2);

match chunks {
    Ok(value) => println(value);
    Err(error) => println(error);
}
```
