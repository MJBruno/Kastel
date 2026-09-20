# Kastel — structure du programme

> État décrit : dépôt `Kastel-corrige` (Kastel 0.1.0, édition Rust 2024, **aucune dépendance externe**).
> ≈ 27 700 lignes de Rust, ≈ 1 500 lignes de Kastel (`std/` + `examples/`).
>
> ⚠️ Cette version n'a pas encore été recompilée après les derniers ajouts (constructeur `initialize`,
> visibilité, `Set`, API standard des collections, opérande `Wide`) : lancer `cargo check` puis `cargo test`.

---

## 1. Vue d'ensemble

Kastel est un langage **dynamique à typage graduel**, compilé en bytecode puis exécuté par une machine virtuelle à pile.

```
  source .ks
      │
      ▼
 ┌─────────┐   tokens   ┌─────────┐    AST     ┌───────────────┐
 │  Lexer  │ ─────────▶ │ Parser  │ ─────────▶ │  TypeChecker  │  (typage graduel, statique)
 └─────────┘            └─────────┘            └───────┬───────┘
                                                       │ AST validé
                                                       ▼
                                               ┌───────────────┐    Function + Chunk
                                               │   Compiler    │ ──────────────────┐
                                               └───────────────┘                   │
                                                                                   ▼
        ┌────────────┐   natives   ┌──────────────┐  Value / Object   ┌────────────────────┐
        │  stdlib    │ ──────────▶ │   runtime    │ ◀──────────────── │  VirtualMachine    │
        │ (Rust)     │             │ (valeurs, GC)│                   │  (pile + frames)   │
        └────────────┘             └──────────────┘                   └─────────┬──────────┘
                                                                                │ import
                                                                                ▼
                                                                     ┌────────────────────┐
                                                                     │  ModuleLoader      │ ─▶ std/*.ks, modules du projet
                                                                     └────────────────────┘
```

Les erreurs de chaque phase (`LexerError`, `ParserError`, `CompileError`, `RuntimeError`) convergent vers
`KastelError`, rendue avec position et suggestions par `Diagnostic`.

---

## 2. Arborescence

```
Kastel/
├── Cargo.toml                 features : debug_trace, trace_gc, profile
├── docs/
│   └── collections-api.md     API standard des collections (figée)
├── examples/                  programmes de démonstration (.ks)
├── std/                       bibliothèque standard écrite en Kastel
└── src/
    ├── main.rs                point d'entrée binaire `kastel`
    ├── lib.rs                 déclare les 8 modules ci-dessous
    ├── app/                   ligne de commande + REPL
    ├── frontend/              lexer, parser, AST
    ├── compiler/              typage statique + génération de bytecode
    ├── bytecode/              Chunk, OpCode, désassembleur
    ├── vm/                    machine virtuelle
    ├── runtime/               valeurs, objets, GC, itérateurs
    ├── stdlib/                fonctions natives et méthodes des types de base
    ├── module/                résolution et chargement des modules
    ├── error/                 types d'erreurs et diagnostics
    └── bin/test_runner.rs     exécuteur de tests de non-régression
```

---

## 3. Modules Rust

### 3.1 `app/` — point d'entrée
| Fichier | Rôle |
|---|---|
| `application.rs` | `Application::run()` : `kastel <fichier.ks>`, REPL sans argument, `--help`, `--version`. `execute()` enchaîne lexer → parser → compiler → VM. Contient aussi le REPL (`repl()`, détection d'entrée incomplète). |

### 3.2 `frontend/` — analyse syntaxique
| Fichier | Rôle |
|---|---|
| `lexer/lexer.rs`, `token.rs` | Découpe en jetons ; mots-clés ; collecte de plusieurs `LexerError`. |
| `ast.rs` | Définition de l'AST : `Expression`, `Statement` (toute instruction est enveloppée dans `Statement::Positioned`), `TypeExpr`, `Pattern`, `ClassField`, `Visibility`, constantes du constructeur (`CONSTRUCTOR_NAME = "initialize"`). |
| `parser/mod.rs` | Outils du parser (`peek`, `check`, `check_next`, `consume`…). |
| `parser/statements.rs` | Instructions, blocs, `let`/`const`, `if`, `while`, `for`, `return`… |
| `parser/expressions.rs` | Précédence des opérateurs, appels, membres, `new`, littéraux `[...]`, `(...)`, `{clé: v}` et `{1, 2, 3}` (ensemble, désucré en `Set(...)`), fonctions anonymes / fléchées. |
| `parser/declarations.rs` | `let`/`const`, annotations de type (`Array<int>`, `Dict<str, int>`, `Tuple<...>`, `Set<T>`). |
| `parser/functions.rs` | `func nom(a: int) -> int { ... }`. |
| `parser/classes.rs` | `class`, `interface`, champs `private let x: int = 0;`, modificateurs `public`/`private` (contextuels), méthodes ; désucre les valeurs initiales de champs en méthode cachée `__fields_<Classe>` ; refuse l'ancien nom `init`. |
| `parser/patterns.rs` | Motifs de `match` et de déstructuration. |
| `parser/exceptions.rs` | `try` / `catch` / `finally`, `throw`. |
| `parser/imports.rs` | `import a.b;`, `from a.b import X, Y;`, `export …`. |

### 3.3 `compiler/` — typage et génération de code
**Typage statique (graduel : `Dynamic` n'est jamais bloquant)**

| Fichier | Rôle |
|---|---|
| `types.rs` | Enum `Type` (`Int`, `Float`, `Str`, `Bool`, `Array<T>`, `Dict<K,V>`, `Tuple`, `Set<T>`, `Function`, `Named`, `Module`, `Dynamic`…), assignabilité, fusion, signatures des méthodes standard de collections. |
| `type_checker.rs` | Vérificateur : inférence, annotations, surcharges par arité, constructeurs, visibilité `private`, types des champs, interfaces, tuples, ensembles, noms de méthodes supprimés. Contient la majorité des tests unitaires de typage. |
| `builtin_types.rs` | Types des fonctions natives globales (`println`, `sqrt`, `range`, `Set`…). |
| `module_types.rs` | Interface de types des modules importés (exports), avec cache. |

**Génération de bytecode**

| Fichier | Rôle |
|---|---|
| `compiler.rs` | Structure `Compiler`, points d'entrée (`compile`, `compile_repl`, `compile_module…`), prédéclaration des globales. |
| `statements.rs` | Compilation des instructions (classes, interfaces, imports, affectations, `match`, `try`…). |
| `expressions.rs` | Compilation des expressions (appels, `InvokeMethod`, membres, tuples…). |
| `declarations.rs`, `functions.rs`, `variables.rs`, `locals.rs`, `scope.rs`, `upvalue.rs`, `context.rs` | Variables locales / globales / upvalues, fonctions et fermetures. |
| `loops.rs`, `control_flow.rs` | Boucles (dont super-instructions), sauts, `break` / `continue`. |
| `emit.rs` | Émission d'octets, **pool de constantes dédoublonné**, indices de constantes sur 16 bits (préfixe `Wide` au-delà de 255), sauts. |

### 3.4 `bytecode/`
| Fichier | Rôle |
|---|---|
| `chunk.rs` | `Chunk` : code, constantes, positions source. |
| `opcode.rs` | `OpCode` (`#[repr(u8)]`, 65 opcodes). `Wide` est le dernier : il préfixe une instruction à opérande constante sur 2 octets. |
| `disassembler.rs` | Affichage lisible du bytecode (mode `debug_trace`). |

### 3.5 `vm/` — machine virtuelle
| Fichier | Rôle |
|---|---|
| `machine.rs` | `VirtualMachine`, `CallFrame`, pile, globales, chargeur de modules. |
| `machine/execution.rs` | Boucle principale (chemin chaud pour les super-instructions), propagation des erreurs. |
| `machine/dispatch.rs` | Dispatch général des opcodes ; `dispatch_wide` pour les opérandes larges. |
| `machine/bytecode.rs` | Lecture des octets, des `u16`, des constantes. |
| `machine/calls.rs`, `closures.rs` | Appels, retours, fermetures, upvalues. |
| `machine/classes.rs` | Création des classes / interfaces, `new` : choix du constructeur par arité, constructeur par défaut implicite, initialisation des champs (base → dérivée), validation des interfaces. |
| `machine/methods.rs` | `InvokeMethod` sur instances, tableaux, dicts, tuples, ensembles, chaînes, `range`, modules ; `base.méthode()` ; **contrôle de visibilité** (`ensure_member_access`). |
| `machine/properties.rs`, `objects.rs`, `arrays.rs`, `tuples.rs`, `variables.rs` | Propriétés, littéraux, indexation, globales. |
| `machine/iterators.rs` | Itérateurs et leurs méthodes (`map`, `filter`, `take`…). |
| `machine/arithmetic.rs` | Arithmétique, comparaisons, opérateurs binaires. |
| `machine/exceptions.rs`, `control_flow.rs`, `modules.rs`, `stack.rs`, `gc.rs`, `debug.rs`, `profiling.rs` | Exceptions, sauts, imports, pile, ramasse-miettes, traces. |

### 3.6 `runtime/` — modèle de valeurs
| Fichier | Rôle |
|---|---|
| `value.rs` | `Value` : `Integer`, `Float`, `Boolean`, `None`, `Range`, `NativeFunction`, `Object(Gc<Object>)`. Égalité, affichage, propriétés, opérations `Dict` / `Tuple` / `Set`. |
| `object.rs` | `Object` : `String`, `Array`, `Tuple`, `Dict`, `Set`, `Function`, `Closure`, `BoundMethod`, `Iterator`, `Module`, `Class` (méthodes surchargées par arité, membres privés), `Interface` (signatures `(nom, arité)`), `Instance`. |
| `gc.rs`, `gc_handle.rs` | Handle `Gc<T>` (`Rc<RefCell<T>>`) + registre et collecte *mark & sweep* pour les cycles. |
| `iterator.rs` | États d'itérateurs (tableau, tuple, ensemble, dict, plage, chaîne, chaînes d'adaptateurs). |
| `function.rs`, `closure.rs`, `upvalue.rs` | Fonctions compilées, fermetures (avec `owner_class`), upvalues. |

### 3.7 `stdlib/` — natives Rust
| Fichier | Contenu |
|---|---|
| `mod.rs` | Enregistrement des natives ; helpers de l'API standard (`to_string_method`, `renamed_method_error`) ; tests de l'API standard. |
| `array.rs`, `dict.rs`, `tuple.rs`, `set.rs`, `string.rs` | Méthodes des types de base (API standard : `size`, `is_empty`, `contains`, `copy`, `clear`, `add`, `remove`, `to_string`…). |
| `iterator.rs` | `range()`, `list()`. |
| `math.rs` | `sqrt`, `pow`, trigonométrie, `rand*`… |
| `io.rs`, `file.rs`, `path.rs`, `os.rs`, `system.rs` | `println`/`input`, fichiers, chemins, système, conversions (`int`, `str`, `type`…). |
| `json.rs`, `debug.rs` | `json_encode` / `json_decode`, `inspect`. |

### 3.8 `module/`
| Fichier | Rôle |
|---|---|
| `resolver.rs` | Résolution déterministe : `std.*` (réservé), local (relatif au fichier), puis racine du projet. Distingue `import a.b` (module) de `import a.B` (export). |
| `module.rs` | `ModuleLoader` (cache, détection de cycles), `ModuleInstance` (globales et exports). |

### 3.9 `error/`
`lex_error.rs`, `parse_error.rs`, `compile_error.rs`, `runtime_error.rs`, `machine_error.rs` → `kastel_error.rs` (`KastelError`) ; `diagnostic.rs` (rendu avec ligne, colonne, aide) ; `suggest.rs` (« Vouliez-vous dire … ? »).

### 3.10 `bin/test_runner.rs`
Exécuteur de non-régression : compile `kastel`, lance les `.ks` d'un dossier de tests (dont `errors/` et `regression/`) et compare aux sorties attendues. Options : `--update` (régénère les sorties), `--clean` (les supprime) ; les deux sont exclusives.

---

## 4. Bibliothèque standard en Kastel (`std/`)

| Module | Contenu |
|---|---|
| `math.ks` | Constantes (`PI`, `E`, `TAU`), natives ré-exportées, hyperboliques, `cbrt`, `hypot`, interpolation, théorie des nombres, combinatoire, statistiques, aléatoire, classe `Complexe`. |
| `collections.ks` | Utilitaires sur collections. |
| `strings.ks` | Utilitaires sur chaînes. |
| `datetime.ks` | Dates et heures. |
| `testing.ks` | Petit cadre d'assertions. |

Import : `import std.math;` (qualifié : `math.sqrt(2.0)`), `import std.math.Complexe;` (export direct), `from std.math import PI, gcd;`.

---

## 5. Fonctionnalités du langage

| Domaine | État |
|---|---|
| Types | Inférence (`let x = 10` → `int`), annotations, `Array<T>`, `Dict<K,V>`, `Tuple<…>`, `Set<T>`, typage graduel (`dynamic`). |
| Fonctions | `func`, paramètres et retour typés, fonctions anonymes, fermetures. |
| Classes | Héritage, interfaces, `base.méthode()`, `this`, `is`. |
| Constructeur | `initialize(...)`, **surchargeable par arité**, constructeur par défaut implicite, hérité des classes de base. |
| Surcharge | Méthodes et constructeurs par arité ; interfaces : signatures `(nom, arité)`. Pas de surcharge de fonctions libres. |
| Champs | `let nom: type = valeur;` dans une classe, initialisés à chaque `new` (base d'abord). |
| Visibilité | `public` (défaut) / `private` sur champs et méthodes ; contrôle statique si le type est connu, **toujours** contrôlé à l'exécution. |
| Collections | `Array`, `Dict`, `Tuple` (immuable), `Set` (éléments uniques), `range`. API unifiée (voir `docs/collections-api.md`). |
| Contrôle | `if`, `while`, `for x in c`, `match` avec motifs, `break`, `continue`, `try/catch/finally`, `throw`. |
| Modules | `import`, `from … import`, `export`. |
| Diagnostics | Positions, suggestions, erreurs guidées pour les noms supprimés (`length`, `push`, `has`, `items`, `to_iterator`, `init`). |

---

## 6. Exemples (`examples/`)

| Fichier | Illustre |
|---|---|
| `types_demo.ks` | Inférence, annotations, tuples typés. |
| `overloads_demo.ks` | Surcharge de méthodes, constructeurs et interfaces. |
| `constructors_demo.ks` | `initialize`, constructeur par défaut, héritage de constructeur. |
| `personne.ks` + `visibility_demo.ks` | Champ privé typé, accès refusé hors de la classe. |
| `set_demo.ks` | `Set` : unicité, opérations ensemblistes, référence / copie. |
| `collections_demo.ks` | API standard des collections. |
| `std_math_demo.ks` | `std.math` : import qualifié et import de classe. |

---

## 7. Commandes

```text
cargo run -q --bin kastel <fichier.ks>     exécuter un programme
cargo run -q --bin kastel                  REPL
cargo run -q --bin kastel -- --help        aide
cargo check                                vérifier la compilation
cargo test                                 tests unitaires (typage, VM, stdlib, opcodes…)
cargo run -q --bin test_runner             tests de non-régression (.ks)
cargo run --features debug_trace ...       trace d'exécution + désassemblage
cargo run --features profile ...           profilage des instructions
cargo run --features trace_gc ...          trace du ramasse-miettes
```

---

## 8. Décisions de conception à retenir

* **Typage graduel** : `Dynamic` est compatible avec tout ; le vérificateur ne bloque que les erreurs certaines.
* **Surcharge par arité uniquement** (le runtime n'a pas de types) : deux signatures de même nom et même arité sont interdites.
* **`initialize` remplace `init`** ; l'ancien nom est refusé avec un message de migration.
* **Champs initialisés par la VM** avant le constructeur, quelle que soit la façon dont il est choisi.
* **Visibilité portée par le nom** : toutes les surcharges d'un même nom ont la même visibilité.
* **Constantes** : pool dédoublonné, indices 16 bits via `Wide` (jusqu'à 65 536 par fragment).
* **`Set` / `Dict`** : listes linéaires comparées par `Value::equals` (pas de hachage) ; tuples comparés par contenu dans un `Set`.
* **Références / copies** : `Array`, `Dict`, `Set` partagés par référence, `copy()` explicite ; `Tuple` immuable.

---

## 9. Limites connues

* Fonctions libres non surchargeables.
* Le contrôle statique de visibilité ne voit pas les classes importées d'un autre module (la VM tranche alors).
* `Set` et `Dict` ont une recherche en O(n).
* Le REPL ne gère pas `import` (pas de chemin source).
* Un `init` privé n'empêche pas `new` ; `json_encode` lit les champs privés directement.
