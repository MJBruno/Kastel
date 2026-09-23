# Records, Dicts, alias de type, unions et `List`

## `Array` devient `List`

Le type `Array<T>` s'appelle désormais **`List<T>`** (`let values: List<int> = [1, 2, 3];`). `Array` est refusé avec un
message explicite. Autres noms qui suivent : `to_array()` (Tuple, Set) devient `to_list()`, et `type([1])` renvoie `"list"`.

## Record : `{ champ: valeur }`

Clés = **identifiants**. Champs nommés, forme **fixe** :

```kastel
let p = { name: "Bruno", age: 25 };

println(p.name);        // Bruno
p.age = 26;             // modifier un champ existant : permis
// p.email = "x";       // ajouter un champ : refusé
```

* Accès et écriture par `p.champ` (pas par `p["champ"]`).
* Objet **partagé** (`let b = a;` désigne le même record) ; `a.copy()` en fabrique un autre.
* Affichage : `{name: "Bruno", age: 26}` (noms sans guillemets) ; `type(p)` vaut `"record"`.
* Méthodes d'introspection : `keys()`, `values()`, `entries()`, `copy()`, `to_string()`. Un **champ** de même nom l'emporte
  (un record peut donc contenir des fonctions : `{ greet: func() { ... } }`).
* Encodé en objet JSON par `json_encode`. Comparé par **identité** avec `==`, comme `List` et `Dict`.
* Typage **structurel** : un record qui a *au moins* les champs attendus, avec des types compatibles, convient.

## Dict : `{ "clé": valeur }`

Clés = **chaînes**, ajout et retrait dynamiques. On lit **par clé** (`d["name"]`, `d.get("name")`) — `d.name` est une erreur
(« Vouliez-vous dire '["name"]' ? »). `{}` reste le dict vide.

**Les deux formes ne se mélangent pas** : `{ name: 1, "age": 2 }` est une erreur de syntaxe, tout comme une clé répétée.

| Littéral | Type | Accès |
|---|---|---|
| `{ name: "Bruno" }` | Record | `p.name` |
| `{ "name": "Bruno" }` | Dict | `d["name"]`, `d.get("name")` |
| `{1, 2, 3}` | Set | `s.contains(2)` |
| `Set()` | Set vide | |

`json_decode` produit un **Dict** (clés arbitraires).

## Alias de type : `type Nom = ...;`

```kastel
type Person = { name: str, age: int };
type Names = List<str>;
type Number = int | float;
```

* Utilisable **avant** sa déclaration (dans la signature d'une fonction, par exemple).
* Peut désigner un type record, une union, un type générique ou un autre alias.
* **Local au fichier** (non exportable pour l'instant) ; n'a aucun effet à l'exécution.
* `type` reste une fonction ordinaire (`type(x)`).

## Union : `A | B`

```kastel
type Number = int | float;

func half(x: Number) -> float { return x / 2; }

let id: str | int = "abc-1";
```

* Une valeur convient si elle convient à **un** membre ; une union convient à une cible si **chaque** membre convient.
* L'arithmétique sur une union est typée membre par membre : `Number + Number` vaut `int | float`.
* Pas de restriction (« narrowing ») par `is` pour l'instant.

## Unions en paramètre de fonction

`int | float` (ou un alias qui s'y résout) fonctionne comme annotation de paramètre, exactement comme pour `let` — y
compris **exportée** : `export func half(n: Number) -> float { return n / 2; }` peut être appelée depuis un autre
module avec un `int` ou un `float`. La vérification est faite par le même mécanisme que pour un type simple, membre
par membre : voir la section « Union » ci-dessus.

## Limites

* Classes et alias ne partagent pas encore de génériques (`type Box<T> = ...`).
* Les classes importées d'un autre module restent typées sans détail.
* Un record se compare par identité (`==`) ; comparer les champs se fait champ par champ.

## Alias exportables entre modules

```kastel
// shapes.ks
export type Point = { x: int, y: int };
export type Number = int | float;

export func origin() -> Point { return { x: 0, y: 0 }; }
```

```kastel
// main.ks
from shapes import Point, Number, origin;   // ou : import shapes.Point;

let p: Point = origin();
let n: Number = 3;
```

* `export type X = ...;` rend l'alias visible aux autres modules, résolu (les alias qu'il référence sont déjà développés) — un alias non exporté reste local au fichier.
* Import ciblé (`from m import X;`), par point (`import m.X;`, comme pour une classe) ou par `from m import *;` : les trois fonctionnent.
* Un alias n'a **aucune existence à l'exécution** : `let leak = Point;` (l'utiliser comme valeur) est refusé — rien n'est défini au runtime sous ce nom, contrairement à une classe importée.
