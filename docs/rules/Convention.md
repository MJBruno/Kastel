# Kastel — Spécification normative des noms de types

**Statut :** Proposition normative
**Version :** 1.0
**Objectif :** fixer définitivement la convention de nommage et de résolution des types Kastel.

---

## 1. Principe fondamental

Kastel utilise des noms de types **case-sensitive**.

Les types builtin possèdent une forme canonique unique.

Exemple :

```kastel
int
float
str
bool
```

Les variantes suivantes ne sont **pas** des alias valides :

```kastel
Int
INT
Float
String
Bool
```

Cette règle permet de conserver un espace de noms clair pour les types utilisateur.

---

# 2. Convention de casse

La convention officielle est :

| Catégorie              | Convention                    | Exemple          |
| ---------------------- | ----------------------------- | ---------------- |
| Type primitif          | `lowercase`                   | `int`            |
| Type spécial           | nom réservé défini par Kastel | `None`           |
| Collection builtin     | `PascalCase`                  | `List<int>`     |
| Type générique builtin | `PascalCase`                  | `Dict<str, int>` |
| Type utilisateur       | `PascalCase` recommandé       | `Personne`       |
| Paramètre générique    | `PascalCase` court            | `T`, `K`, `V`    |
| Type interne Rust      | convention Rust               | `Type::Int`      |

---

# 3. Types primitifs

Les types primitifs officiels sont :

```text
int
float
bool
str
```

## 3.1 `int`

Entier signé 64 bits.

Représentation runtime :

```rust
Value::Integer(i64)
```

Type interne :

```rust
Type::Int
```

Syntaxe :

```kastel
let age: int = 26
```

Formes interdites :

```kastel
let age: Int = 26
let age: INT = 26
let age: integer = 26
let age: Integer = 26
```

---

## 3.2 `float`

Nombre flottant double précision.

Représentation runtime :

```rust
Value::Float(f64)
```

Type interne :

```rust
Type::Float
```

Syntaxe :

```kastel
let pi: float = 3.14159
```

Formes interdites :

```kastel
Float
FLOAT
double
Double
```

---

## 3.3 `bool`

Booléen.

Valeurs :

```kastel
true
false
```

Type :

```kastel
bool
```

Type interne :

```rust
Type::Bool
```

---

## 3.4 `str`

Chaîne Unicode.

Type :

```kastel
str
```

Type interne :

```rust
Type::Str
```

Kastel ne doit pas utiliser `string` comme forme canonique.

Donc :

```kastel
let name: str = "Bruno"
```

est correct.

```kastel
let name: string = "Bruno"
```

doit être considéré comme invalide dans la syntaxe normative.

Cela évite d'avoir deux noms officiels pour le même type.

---

# 4. `None`

`None` représente l'absence de valeur.

Valeur :

```kastel
None
```

Type :

```kastel
None
```

Type interne :

```rust
Type::None
```

Exemple :

```kastel
let result: None = None
```

Cependant, dans la majorité des API, `None` doit apparaître dans une union :

```kastel
T | None
```

Exemple :

```kastel
let value: int | None = List.pop()
```

---

# 5. `Dynamic`

Kastel possède un type spécial représentant une valeur dont le type statique n'est pas déterminé :

```text
dynamic
```

Recommandation : la forme utilisateur officielle doit être **`dynamic` en minuscules**.

Type interne :

```rust
Type::Dynamic
```

Exemple :

```kastel
let value: dynamic = get_value()
```

`dynamic` est un mécanisme d'échappement du système de typage, pas un type concret classique.

---

# 6. Ne pas introduire `Any`

Kastel ne doit pas avoir simultanément :

```text
dynamic
Any
any
Dynamic
```

Ces concepts seraient trop proches.

La spécification utilise uniquement :

```text
dynamic
```

pour le type dynamique explicite.

En interne :

```rust
Type::Dynamic
```

---

# 7. Types de collections

Les collections builtin utilisent **PascalCase**.

Cela les distingue clairement des primitifs.

Types officiels :

```text
List<T>
Dict<K, V>
Set<T>
Tuple<T, ...>
```

---

# 8. `List<T>`

Le type tableau officiel est :

```text
List<T>
```

et non :

```text
List<T>
list<T>
List<T>
```

Exemples :

```kastel
let numbers: List<int> = [1, 2, 3]
let names: List<str> = ["Bob", "Alice"]
let values: List<float> = [1.0, 2.0, 3.0]
```

Type interne :

```rust
Type::List(Box<Type>)
```

ou équivalent dans l'implémentation courante.

---

# 9. `Dict<K, V>`

Le dictionnaire officiel est :

```text
Dict<K, V>
```

Exemple :

```kastel
let ages: Dict<str, int> = {
    "Alice": 25,
    "Bob": 30
}
```

Convention :

```text
K = key
V = value
```

Type interne :

```rust
Type::Dict(...)
```

---

# 10. `Set<T>`

L'ensemble officiel est :

```text
Set<T>
```

Exemple :

```kastel
let values: Set<int> = {1, 2, 3}
```

Type interne :

```rust
Type::Set(...)
```

Le type élément doit être conservé statiquement :

```text
Set<int>
Set<str>
Set<float>
```

---

# 11. `Tuple`

Le tuple utilise :

```text
Tuple<T1, T2, ...>
```

Exemple :

```kastel
let point: Tuple<int, int> = (10, 20)
```

Tuple hétérogène :

```kastel
let user: Tuple<str, int, bool> = ("Bruno", 26, true)
```

Le tuple doit conserver le type de chaque position.

Ainsi :

```text
Tuple<int, str>
```

n'est pas équivalent à :

```text
Tuple<str, int>
```

---

# 12. Tuple vide

Le tuple vide :

```kastel
()
```

peut être représenté comme :

```text
Tuple<>
```

ou par un type interne dédié si cela simplifie le runtime.

La notation utilisateur officielle reste :

```kastel
()
```

---

# 13. Types de tableaux imbriqués

Les types génériques doivent pouvoir être imbriqués :

```kastel
List<List<int>>
```

```kastel
Dict<str, List<int>>
```

```kastel
Set<Tuple<int, str>>
```

Exemple :

```kastel
let matrix: List<List<int>>
```

---

# 14. Classes utilisateur

Les classes définies par l'utilisateur utilisent leur nom propre.

Exemple :

```kastel
class Personne {}
class Animal {}
class Dog: Animal {}
```

Le type de :

```kastel
let p: Personne
```

est :

```text
Personne
```

et non :

```text
class Personne
Class<Personne>
```

---

# 15. Interfaces

Les interfaces sont utilisées directement comme types.

Exemple :

```kastel
interface Animal {
    func speak() -> str;
}
```

Une variable peut avoir :

```kastel
let animal: Animal
```

Le nom `Animal` désigne alors le contrat d'interface.

Il ne faut pas ajouter une syntaxe artificielle telle que :

```text
Interface<Animal>
```

---

# 16. Héritage

Une classe dérivée possède son propre nom de type.

Exemple :

```kastel
class Animal {}
class Dog: Animal {}
```

Alors :

```text
Dog
Animal
```

sont deux types distincts.

La relation :

```text
Dog <: Animal
```

est une relation d'assignabilité, pas une fusion des noms.

---

# 17. Types génériques utilisateur

Lorsqu'un système de génériques complet sera introduit, les paramètres génériques utiliseront généralement des identifiants courts en PascalCase :

```text
T
K
V
E
R
```

Exemple :

```kastel
class Box<T> {
    value: T;
}
```

Le type :

```text
Box<int>
```

est donc valide.

Autres exemples :

```text
Pair<int, str>
Result<int, str>
Map<str, int>
```

---

# 18. Règle sur `T`, `K`, `V`

Convention recommandée :

```text
T = type principal
U = second type générique
V = troisième type
K = clé
V = valeur
E = élément
R = résultat
```

Exemple :

```kastel
class Box<T>
class Pair<T, U>
class Dict<K, V>
```

Cette convention est documentaire ; elle ne doit pas devenir une contrainte du compilateur.

---

# 19. Types union

Kastel doit utiliser :

```text
T | U
```

pour une union.

Exemple :

```kastel
int | str
```

ou :

```kastel
List<int> | None
```

ou :

```kastel
Dog | Cat
```

## Associativité

Une union :

```text
A | B | C
```

doit être représentée de manière canonique en interne.

Par exemple :

```text
Union<A, B, C>
```

plutôt que des arbres dépendants de l'ordre de parsing.

---

# 20. `None` dans les unions

C'est le modèle recommandé pour les API qui peuvent ne rien retourner :

```text
T | None
```

Exemples :

```text
int | None
str | None
List<int> | None
Personne | None
```

Ainsi :

```kastel
List.pop()
```

pourrait avoir :

```text
T | None
```

au lieu de simplement :

```text
T
```

---

# 21. Types imbriqués et unions

Les unions doivent être valides dans les paramètres génériques :

```kastel
List<int | None>
```

```kastel
Dict<str, int | float>
```

```kastel
Set<int | float>
```

Toutefois, les règles de variance et de mutabilité doivent être définies séparément.

---

# 22. `Range`

Le type utilisateur officiel doit être :

```text
Range
```

ou, lorsqu'un système générique de ranges est introduit :

```text
Range<int>
```

Pour l'état actuel de Kastel, la recommandation est :

```text
Range
```

avec une sémantique entière :

```text
start: int
stop: int
step: int
```

Important :

```text
Range ne doit pas utiliser float.
```

Donc :

```text
Range → éléments int
```

et non :

```text
Range → float
```

---

# 23. Types littéraux

Les littéraux suivants ont les types :

| Littéral   | Type              |
| ---------- | ----------------- |
| `123`      | `int`             |
| `3.14`     | `float`           |
| `true`     | `bool`            |
| `false`    | `bool`            |
| `"hello"`  | `str`             |
| `'hello'`  | `str`             |
| `None`     | `None`            |
| `[1, 2]`   | `List<int>`      |
| `{1, 2}`   | `Set<int>`        |
| `(1, "a")` | `Tuple<int, str>` |

---

# 24. Inférence des collections

Les collections littérales doivent être inférées lorsque c'est possible.

Exemple :

```kastel
let numbers = [1, 2, 3]
```

donne :

```text
List<int>
```

Exemple :

```kastel
let names = ["Alice", "Bob"]
```

donne :

```text
List<str>
```

Exemple :

```kastel
let values = [1, 2.5]
```

doit produire un type correspondant au système d'union/numeric promotion retenu.

Le système ne doit pas immédiatement perdre l'information en :

```text
dynamic
```

lorsqu'une union précise est représentable.

---

# 25. `Record`

Pour un objet littéral structuré avec des champs connus, le type recommandé est :

```text
Record<...>
```

mais la syntaxe exacte de type doit être fixée séparément.

Exemple conceptuel :

```kastel
type User = {
    name: str,
    age: int
}
```

Dans ce cas :

```text
User
```

devient le nom du type alias.

Un objet anonyme ne doit pas être confondu avec :

```text
Dict<str, dynamic>
```

car un record possède des champs connus et nommés.

---

# 26. Type alias

Un alias doit utiliser le nom défini par l'utilisateur.

Exemple :

```kastel
type UserId = int
```

Puis :

```kastel
let id: UserId
```

Le nom :

```text
UserId
```

doit rester case-sensitive.

Ainsi :

```text
UserId
userid
USERID
```

sont trois identifiants différents.

---

# 27. Noms builtin réservés

Les noms suivants doivent être réservés :

```text
int
float
bool
str
None
dynamic

List
Dict
Set
Tuple
Range
```

À cela peuvent s'ajouter les builtin futurs.

Ils ne devraient pas pouvoir être redéfinis comme types locaux :

```kastel
class int {}
type str = ...
```

Le compilateur doit produire une erreur de conflit de nom.

---

# 28. Ne pas rendre les builtin case-insensitive

Cette logique actuelle :

```rust
name.to_ascii_lowercase()
```

ne doit pas être utilisée pour résoudre les noms de types Kastel.

À la place :

```text
int     → builtin int
Int     → identifiant utilisateur normal
INT     → identifiant utilisateur normal
```

Si `Int` n'existe pas comme type utilisateur :

```text
Unknown type: Int
```

---

# 29. Exemple de résolution

Pour :

```kastel
let a: int
let b: Int
let c: Personne
let d: Dynamic
```

la résolution doit être :

```text
a -> builtin int
b -> lookup utilisateur "Int"
c -> lookup utilisateur "Personne"
d -> lookup utilisateur "Dynamic"
```

Mais `Dynamic` n'est pas la forme canonique utilisateur.

La forme correcte est :

```kastel
let d: dynamic
```

---

# 30. Canonicalisation interne

Les types doivent être comparés par leur représentation interne, jamais par le texte original.

Exemple :

```text
int
```

devient :

```rust
Type::Int
```

Puis toutes les comparaisons utilisent :

```rust
Type::Int == Type::Int
```

et jamais :

```text
"int" == "INT"
```

Le texte n'est utilisé que pour le parsing et les diagnostics.

---

# 31. Affichage canonique

`Display` du type interne doit toujours produire la forme normative Kastel.

Exemple :

```rust
Type::Int       -> "int"
Type::Float     -> "float"
Type::Bool      -> "bool"
Type::Str       -> "str"
Type::None      -> "None"
Type::Dynamic   -> "dynamic"
```

Collection :

```text
List<int>
Dict<str, int>
Set<float>
Tuple<int, str>
```

Classe :

```text
Personne
```

Interface :

```text
Animal
```

Union :

```text
int | str
```

---

# 32. Diagnostics

Les erreurs du compilateur doivent utiliser les noms canoniques.

Exemple :

```text
Type mismatch:
expected int
found str
```

et non :

```text
expected Int
found String
```

Même lorsque le développeur manipule des noms erronés.

---

# 33. Suggestions d'erreur

Les suggestions peuvent proposer une forme canonique.

Exemple :

```kastel
let x: Int = 10
```

Diagnostic :

```text
Unknown type 'Int'.
Did you mean 'int'?
```

Cela est préférable à l'acceptation silencieuse de `Int`.

---

# 34. Conventions interdites

Les formes suivantes ne sont pas des aliases builtin :

```text
Int
Integer

Float
Double

Bool
Boolean

String
string

NoneType
Nil
nil
Null
null

Any
any

List
list
```

Elles peuvent éventuellement être des types utilisateur si le langage autorise ces noms dans un espace de noms non réservé, mais elles ne désignent pas automatiquement les builtin Kastel.

---

# 35. Tableau définitif

| Type Kastel  | Forme canonique | Rust interne                |
| ------------ | --------------- | --------------------------- |
| entier       | `int`           | `Type::Int`                 |
| flottant     | `float`         | `Type::Float`               |
| booléen      | `bool`          | `Type::Bool`                |
| chaîne       | `str`           | `Type::Str`                 |
| absence      | `None`          | `Type::None`                |
| dynamique    | `dynamic`       | `Type::Dynamic`             |
| tableau      | `List<T>`      | `Type::List<T>`            |
| dictionnaire | `Dict<K,V>`     | `Type::Dict<...>`           |
| ensemble     | `Set<T>`        | `Type::Set<T>`              |
| tuple        | `Tuple<T,...>`  | `Type::Tuple<...>`          |
| range        | `Range`         | `Type::Range`               |
| union        | `A \| B`        | `Type::Union<...>`          |
| classe       | `Personne`      | `Type::Named(...)` / classe |
| interface    | `Animal`        | type nominal                |
| alias        | `UserId`        | type nommé / alias          |
| générique    | `Box<T>`        | type paramétrique           |

---

# 36. Règle finale

La règle normative de Kastel est donc :

```text
PRIMITIFS
─────────
int
float
bool
str

SPÉCIAL
───────
None
dynamic

COLLECTIONS
───────────
List<T>
Dict<K, V>
Set<T>
Tuple<T, ...>
Range

COMPOSITIONS
────────────
A | B

UTILISATEUR
───────────
Personne
Animal
UserId
Box<T>
```

Et la règle la plus importante :

```text
Kastel : case-sensitive
Rust   : PascalCase pour les variantes enum
```

Donc :

```text
Kastel     Rust
------     ----------------
int        Type::Int
float      Type::Float
bool       Type::Bool
str        Type::Str
None       Type::None
dynamic    Type::Dynamic
List<T>    Type::List<T>
Dict<K,V>  Type::Dict<K,V>
Set<T>     Type::Set<T>
Tuple<...> Type::Tuple<...>
```

**Aucune conversion automatique de casse ne doit exister pour résoudre les noms de types Kastel.**