## Formats supportés

| Type      | Formats                                                                                                                         |
| --------- | ------------------------------------------------------------------------------------------------------------------------------- |
| `Integer` | `{}`, `{:d}`, `{:+d}`, `{:08d}`, `{:b}`, `{:o}`, `{:x}`, `{:X}`, `{:#b}`, `{:#x}`, `{:,}`, `{:_}`, `{:<10}`, `{:>10}`, `{:^10}` |
| `Float`   | `{}`, `{:.4}`, `{:.2f}`, `{:.2e}`, `{:.2E}`, `{:.4g}`, `{:.4G}`, `{:.2%}`, `{:+010.2f}`, groupement `,` / `_`                   |
| `String`  | `{}`, `{!s}`, `{!r}`, `{:10}`, `{:>10}`, `{:^10}`, `{:.5s}`, remplissage personnalisé comme `{:*^10}`                           |
| `Array`   | `{}`, `{:>20}`, `{:^20}`, `{:20}`, précision de représentation `"{:.20}"`                                                       |
| `Dict`    | mêmes possibilités d'affichage, alignement et largeur                                                                           |
| `Tuple`   | mêmes possibilités d'affichage, alignement et largeur                                                                           |


```javaScript
println("|{:10}|", "Kastel");
println("|{:>10}|", "Kastel");
println("|{:^10}|", "Kastel");
println("|{:*^10}|", "Kastel");
println("|{:.5s}|", "Kastel");
```
ça donne:
```javascript
|Kastel    |
|    Kastel|
|  Kastel  |
|**Kastel**|
|Kaste|
```

### Pour les nombre:

```javascript
println("|{:10}|", 42);
println("|{:>10}|", 42);
println("|{:^10}|", 42);
println("|{:010}|", 42);
```

```javascript
|        42|
|        42|
|    42    |
|0000000042|
```
*Important : {:10} ne signifie pas "transformer la chaîne en 10 caractères exactement". C'est minimum 10.*