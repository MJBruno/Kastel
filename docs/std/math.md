# std.math

Mathématiques numériques, primitives natives et utilitaires sûrs.

## Erreurs récupérables

Les opérations qui peuvent échouer avec les valeurs fournies utilisent `Result`, notamment `safe_sqrt`, `safe_log`, `safe_log10`, `safe_asin`, `safe_acos`, `safe_div`, `safe_idiv` et `safe_pow`.

## Génériques

`clamp`, `min_of`, `max_of`, `min_list`, `max_list` et `sum` utilisent les capabilities officielles `Ord` et `Add` de `std.ops`.
