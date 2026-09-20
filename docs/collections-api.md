# API standard des collections

Une seule convention pour `Array`, `Dict`, `Tuple`, `Set`, `String` et `Range`.

| Famille      | Méthodes                                              |
|--------------|-------------------------------------------------------|
| Taille       | `size()`, `is_empty()`                                |
| Recherche    | `contains(x)`, `index_of(x)`                          |
| Mutation     | `add(x)`, `remove(x)`, `clear()`                      |
| Copie        | `copy()`                                              |
| Conversion   | `to_array()`, `to_string()`                           |
| Parcours     | `iter()` (la syntaxe principale reste `for x in c`)   |
| Dictionnaire | `get(k)`, `set(k, v)`, `keys()`, `values()`, `entries()` |
| Ensembles    | `union`, `intersection`, `difference`, `symmetric_difference`, `is_subset`, `is_superset` |

## Qui expose quoi

| Type   | Communes                                           | Spécifiques |
|--------|----------------------------------------------------|-------------|
| Array  | size, is_empty, contains, copy, clear, to_string   | add, remove, remove_at, pop, insert, get, set, first, last, index_of, slice, reverse, join, sort, map, filter, reduce, any, all |
| Dict   | size, is_empty, contains, copy, clear, to_string   | get, set, remove, keys, values, entries, get_or, update |
| Tuple  | size, is_empty, contains, to_string                | get, first, last, index_of, to_array |
| Set    | size, is_empty, contains, copy, clear, to_string   | add, remove, union, intersection, difference, symmetric_difference, is_subset, is_superset, equals, to_array |
| String | size, is_empty, contains, to_string                | opérations texte (upper, lower, trim, split, ...) |
| Range  | size, is_empty, to_string                          | start, stop, step, + méthodes d'itérateur |

Toutes les méthodes ne sont **pas** offertes à tous les types : `Tuple` est
immuable (ni `add`, ni `remove`, ni `clear`, ni `copy`), et il n'y a pas de
pseudo-interface « tout objet a `copy()` ».

## Règles de sémantique

* `size()` d'une chaîne = nombre de **caractères Unicode** (`"é".size()` et
  `"😀".size()` valent 1), pas d'octets UTF-8.
* `contains(x)` : pour `Dict`, teste l'existence d'une **clé**.
* `add(x)` : renvoie `true` si l'élément a été ajouté. Toujours `true` pour
  `Array` (doublons permis) ; `false` pour un doublon dans un `Set`.
* `remove(x)` : retire par **valeur** (première occurrence pour `Array`) et
  renvoie `true` si l'élément était présent — jamais d'erreur pour un élément
  absent. Pour retirer un `Array` par **position** : `remove_at(index)`, qui
  renvoie l'élément retiré. (`Dict.remove(key)` renvoie la valeur et échoue si
  la clé est absente.)
* Référence / copie : `Array`, `Dict` et `Set` sont des références (`let b = a;`
  partage l'objet) ; `copy()` fabrique un nouvel objet. `Tuple` est une valeur
  immuable.
* `to_string()` produit le rendu de `println`.

## Noms supprimés

Ils échouent avec « Vouliez-vous dire '…' ? » (ou, si le type est connu, avec
une erreur de compilation).

| Ancien nom             | Nouveau nom        |
|------------------------|--------------------|
| `x.length` / `length()`| `x.size()`         |
| `array.push(x)`        | `array.add(x)`     |
| `array.remove(index)`  | `array.remove_at(index)` (et `remove(x)` retire maintenant une **valeur**) |
| `dict.has(key)`        | `dict.contains(key)` |
| `dict.items()`         | `dict.entries()`   |
| `x.to_iterator()`      | `x.iter()`         |
