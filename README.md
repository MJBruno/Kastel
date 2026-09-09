
[![Rust](https://img.shields.io/badge/Rust-2024-orange?style=for-the-badge\&logo=rust)](https://www.rust-lang.org/)
[![License MIT](https://img.shields.io/badge/License-MIT-blue?style=for-the-badge)](LICENSE)
[![Bytecode VM](https://img.shields.io/badge/Bytecode-VM-brightgreen?style=for-the-badge)](src/vm/)

# 🏰 Kastel

> Un langage de programmation dynamique, compilé en bytecode et exécuté par une machine virtuelle stack-based.

**Kastel** est un langage de programmation moderne écrit en **Rust**.

Il repose sur une architecture :

```text
Source Kastel
     ↓
Lexer / Parser
     ↓
AST
     ↓
Compiler
     ↓
Bytecode
     ↓
Stack-Based VM
     ↓
Runtime / GC / Native
```

Kastel est conçu autour d'un objectif simple :

> **Une syntaxe dynamique et expressive avec un runtime structuré, testable et extensible.**

Le projet est actuellement dans une phase de **stabilisation et de consolidation du runtime**.

L'objectif actuel n'est plus uniquement d'ajouter des fonctionnalités, mais de garantir la cohérence du compilateur, du bytecode, de la VM, des closures, des upvalues, du Garbage Collector, des modules et de la bibliothèque standard.

---

# ✨ Fonctionnalités

| Domaine          | Fonctionnalités                                               |
| ---------------- | ------------------------------------------------------------- |
| Langage          | Typage dynamique                                              |
| Variables        | `let`, `const`                                                |
| Fonctions        | Fonctions, récursivité, fonctions imbriquées                  |
| Closures         | Closures, captures lexicales, upvalues                        |
| Collections      | Tableaux dynamiques, objets                                   |
| Contrôle         | `if`, boucles, `for .. in`, `break`, `continue`               |
| Itération        | Itérateurs, `range()`                                         |
| Opérateurs       | Arithmétiques, comparaisons, bitwise                          |
| Modules          | `import`, `export`                                            |
| Compilation      | Compilation en bytecode                                       |
| Runtime          | VM stack-based                                                |
| Mémoire          | Garbage Collector                                             |
| Standard Library | I/O, math, strings, arrays, objects, iterators, system, debug |
| Debug            | Désassembleur bytecode, VM trace, GC trace                    |
| Erreurs          | Erreurs de compilation et erreurs runtime avec localisation   |

---

# 🧠 Architecture

Kastel est organisé en plusieurs couches spécialisées :

```text
                    Source Kastel
                         │
                         ▼
                  ┌─────────────┐
                  │    Lexer    │
                  └──────┬──────┘
                         ▼
                  ┌─────────────┐
                  │    Parser   │
                  └──────┬──────┘
                         ▼
                  ┌─────────────┐
                  │     AST     │
                  └──────┬──────┘
                         ▼
                  ┌─────────────┐
                  │  Compiler   │
                  └──────┬──────┘
                         ▼
                  ┌─────────────┐
                  │   Bytecode  │
                  └──────┬──────┘
                         ▼
                  ┌─────────────┐
                  │     VM      │
                  └──────┬──────┘
                         │
          ┌──────────────┼──────────────┐
          ▼              ▼              ▼
       Runtime         Native         Modules
          │
          ▼
          GC
```

## Organisation du code

```text
src/
├── app/
├── bytecode/
├── compiler/
├── error/
├── frontend/
├── module/
├── native/
├── runtime/
└── vm/
```

### Responsabilités

```text
app
    → démarrage et orchestration de l'application

frontend
    → lexer, parser et AST

compiler
    → résolution des variables et génération du bytecode

bytecode
    → opcodes, chunks, constantes et désassemblage

vm
    → exécution du bytecode

runtime
    → Value, Object, fonctions, closures, upvalues et GC

native
    → bibliothèque standard native de Kastel

module
    → chargement et gestion des modules

error
    → erreurs de compilation et erreurs runtime
```

---

# ⚙️ Machine virtuelle

Kastel utilise une **machine virtuelle basée sur une pile**.

Les fonctions sont exécutées dans des `CallFrame`.

```text
CallFrame
├── closure
├── instruction pointer (ip)
└── slot_start
```

La pile contient notamment :

```text
Stack
 │
 ├── callee
 ├── arguments
 ├── locals
 └── temporary values
```

Les appels suivent cette organisation :

```text
                 Stack
                   │
        ┌──────────┼──────────┐
        ▼          ▼          ▼
      callee      args       locals
                              │
                              ▼
                          CallFrame
```

La VM valide notamment :

```text
stack access
constant access
local slots
upvalue slots
jump targets
function calls
argument counts
opcodes
```

L'objectif est qu'un état invalide du bytecode soit transformé en **`RuntimeError`** plutôt qu'en `panic!` Rust.

---

# 🔗 Closures et Upvalues

Kastel supporte les **closures lexicales** et les **upvalues**.

La compilation utilise une représentation permettant de déterminer si une variable capturée provient :

```text
local
   │
   └──► upvalue

upvalue
   │
   └──► upvalue
```

Au runtime, une upvalue peut être ouverte :

```text
Closure
   │
   ▼
Upvalue
   │
   ▼
Stack slot
```

Puis fermée lorsque la variable locale n'est plus présente dans la stack :

```text
Closure
   │
   ▼
Upvalue
   │
   ▼
closed Value
```

Cela permet notamment :

```JavaScript
function make_counter() {
    let count = 0;

    function increment() {
        count += 1;
        return count;
    }

    return increment;
}

let counter = make_counter();

println(counter());
println(counter());
println(counter());
```

Résultat :

```JavaScript
1
2
3
```

---

# 🧹 Garbage Collector

Kastel utilise un Garbage Collector basé sur le **marquage et le balayage des objets atteignables**.

Le cycle général est :

```text
Roots
  │
  ▼
MARK
  │
  ▼
Reachable Objects
  │
  ▼
SWEEP
  │
  ▼
Reclaim
```

Les structures runtime peuvent notamment contenir :

```text
Object
Array
Dict
Function
Closure
Iterator
Module
```

Le GC doit également suivre les références contenues dans :

```text
Stack
Globals
CallFrames
OpenUpvalues
Function constants
Closed upvalues
Modules
```

Les constantes d'une fonction peuvent elles-mêmes référencer des objets :

```text
Function
   │
   ▼
Chunk
   │
   ▼
Constants
   │
   └──────► Object
```

Les upvalues fermées sont également conservées pendant le tracing :

```text
Upvalue
   │
   ▼
closed Value
```

Le Garbage Collector fait actuellement partie des composants prioritaires pour les tests de régression et l'audit de robustesse.

---

# 📦 Bibliothèque standard

La bibliothèque standard native de Kastel est organisée par domaine :

```text
src/native/
├── mod.rs
├── io.rs
├── math.rs
├── string.rs
├── array.rs
├── object.rs
├── iterator.rs
├── system.rs
└── debug.rs
```

## I/O

Fonctions principales :

```JavaScript
print(...)
println(...)
input(...)
```

## Mathématiques

Fonctions disponibles ou prévues :

```JavaScript
abs(x)
floor(x)
ceil(x)
round(x)

sqrt(x)
pow(x, y)

min(a, b)
max(a, b)

sin(x)
cos(x)
tan(x)

log(x)
log10(x)
exp(x)

rand()
rand_int(max)
rand_range(start, end)
```

Les fonctions aléatoires suivent :

```JavaScript
rand()              → [0, 1)
rand_int(max)       → [0, max)
rand_range(a, b)    → [a, b)
```

## Strings

```JavaScript
format(...)
strlen(value)
lower(value)
upper(value)
trim(value)

starts_with(value, prefix)
ends_with(value, suffix)
string_contains(value, search)

replace(value, from, to)
split(value, separator)
```

## Arrays

```JavaScript
push(array, value)
pop(array)
length(array)
insert(array, index, value)
remove(array, index)

array_contains(array, value)
clear(array)
```

## Objects

```JavaScript
object()

object_get(object, key)
object_set(object, key, value)

object_has(object, key)
object_keys(object)
object_values(object)
object_length(object)
```

## Iterateurs

```JavaScript
range(...)
list(iterator)
```

`range()` est conçu comme un itérateur lazy.

Exemple :

```JavaScript
for i in range(5) {
    println(i);
}
```

Résultat :

```JavaScript
0
1
2
3
4
```

## System

```JavaScript
int(value)
float(value)
str(value)
bool(value)
type(value)

clock()
cwd()
env(name)
```

## Debug

```JavaScript
inspect(value)
debug(value)
```

---

# 🔁 Itération

Kastel privilégie une syntaxe `for .. in` :

```JavaScript
for value in [10, 20, 30] {
    println(value);
}
```

Avec `range()` :

```JavaScript
for i in range(5) {
    println(i);
}
```

Plage personnalisée :

```JavaScript
for i in range(2, 10, 2) {
    println(i);
}
```

Résultat :

```JavaScript
2
4
6
8
```

Itération descendante :

```JavaScript
for i in range(10, 0, -1) {
    println(i);
}
```

---

# 📦 Modules

Kastel possède un système de modules permettant de séparer les programmes en plusieurs fichiers.

Les opérations principales sont :

```JavaScript
import ...
export ...
```

Le runtime utilise un `ModuleLoader` pour charger les modules.

L'objectif du système est de fournir :

```text
module resolution
module loading
module caching
exports
runtime isolation
module errors
```

Le système de modules fait partie des composants à consolider avec la stabilisation du runtime.

---

# 🛡️ Gestion des erreurs

Kastel distingue les erreurs de compilation des erreurs d'exécution.

Exemples d'erreurs runtime :

```text
TypeError
DivisionByZero
WrongArgumentCount
NotCallable
IndexOutOfBounds
NotIndexable
NotObject
NotIterable
IteratorExhausted
InvalidOpcode
StackUnderflow
ModuleError
```

Les erreurs peuvent conserver leur emplacement dans le programme :

```text
ligne X, colonne Y : message
```

Principe général :

```text
Programme Kastel invalide
        │
        ▼
    RuntimeError
        │
        ▼
    Diagnostic
```

Le runtime doit éviter les `panic!` pour les erreurs provenant du programme ou du bytecode.

---

# 🧪 Tests

Kastel possède une suite de tests couvrant progressivement les différents niveaux du langage et du runtime :

```text
Lexer
Parser
Compiler
Bytecode
VM
Functions
Closures
Upvalues
Arrays
Objects
Iterators
Modules
GC
Runtime Errors
Standard Library
```

Commandes principales :

```bash
cargo check
cargo test
```

Le principe de développement est :

```text
Bug
 ↓
Correction
 ↓
Test de régression
 ↓
Validation
 ↓
Fonctionnalité stabilisée
```

Les programmes de test Kastel sont regroupés dans :

```text
test/
```

---

# 🔍 Debug et instrumentation

Kastel possède des fonctionnalités de traçage pour faciliter le développement du runtime.

## VM trace

```bash
cargo run --features debug_trace -- examples/main.ks
```

## GC trace

```bash
cargo run --features trace_gc -- examples/main.ks
```

## VM + GC

```bash
cargo run --features "debug_trace,trace_gc" -- examples/main.ks
```

Ces fonctionnalités permettent notamment d'observer :

```text
bytecode
instructions
stack
calls
GC allocations
GC marking
GC sweeping
```

---

# 🚧 État du projet

Kastel est actuellement en **phase de consolidation du runtime et de la bibliothèque standard**.

L'architecture principale est en place :

```text
Lexer / Parser
      ↓
Compiler
      ↓
Bytecode
      ↓
VM
      ↓
Runtime
      ↓
GC
```

Les mécanismes principaux déjà présents comprennent :

```text
✅ Typage dynamique
✅ Bytecode
✅ Stack-Based VM
✅ Variables locales et globales
✅ const
✅ Fonctions
✅ Récursivité
✅ Closures
✅ Upvalues
✅ Arrays
✅ Objects
✅ Iterators
✅ range()
✅ for .. in
✅ Modules
✅ Garbage Collector
✅ Runtime errors
✅ Native functions
```

## Priorités actuelles

```text
1.  Finaliser la bibliothèque standard
2.  Tester systématiquement la stdlib
3.  Renforcer les tests de régression
4.  Auditer le Compiler
5.  Auditer la VM
6.  Auditer le Garbage Collector
7.  Consolider le système de modules
8.  Uniformiser les diagnostics
9.  Refactoriser les composants trop volumineux
10. Ajouter des benchmarks
11. Optimiser après mesure
12. Documenter la spécification du langage
```

Le projet privilégie désormais la **stabilité des mécanismes existants** avant l'ajout de fonctionnalités majeures.

---

# 🛠️ Installation

## Prérequis

* Rust
* Cargo

Kastel utilise actuellement l'édition **Rust 2024**.

Cloner le dépôt :

```bash
git clone https://github.com/MJBruno/Kastel.git
cd Kastel
```

---

# 🔨 Compilation

Compilation normale :

```bash
cargo build
```

Compilation release :

```bash
cargo build --release
```

Vérification :

```bash
cargo check
```

Tests :

```bash
cargo test
```

---

# ▶️ Exécuter un programme Kastel

```bash
cargo run -- examples/main.ks
```

Exemple :

```bash
cargo run -- examples/main.ks
```

---

# 📁 Structure du projet

```text
Kastel/
├── examples/
│
├── src/
│   ├── app/
│   ├── bytecode/
│   ├── compiler/
│   ├── error/
│   ├── frontend/
│   ├── module/
│   ├── native/
│   │   ├── mod.rs
│   │   ├── io.rs
│   │   ├── math.rs
│   │   ├── string.rs
│   │   ├── array.rs
│   │   ├── object.rs
│   │   ├── iterator.rs
│   │   ├── system.rs
│   │   └── debug.rs
│   ├── runtime/
│   └── vm/
│
├── test/
├── Cargo.toml
├── Cargo.lock
├── LICENSE
└── README.md
```

---

# 🎯 Philosophie du projet

Kastel suit une idée centrale :

> **Un langage simple en surface, avec un runtime conçu pour être solide et compréhensible.**

Le projet cherche à maintenir une séparation claire entre :

```text
Language
    ↓
Compiler
    ↓
Bytecode
    ↓
VM
    ↓
Runtime
    ↓
GC
```

Cette séparation doit permettre au langage d'évoluer sans transformer le runtime en composant monolithique.

---

# 🗺️ Évolution prévue

À moyen terme, le développement suivra principalement cette trajectoire :

```text
                    Kastel
                       │
        ┌──────────────┼──────────────┐
        ▼              ▼              ▼
     Language        Runtime        Stdlib
        │              │              │
        ▼              ▼              ▼
     Syntax          VM / GC       Native API
        │              │              │
        └──────────────┼──────────────┘
                       ▼
                 Tests / Benchmarks
                       │
                       ▼
                 Stabilisation
                       │
                       ▼
                    Kastel 0.1
```

L'objectif est de construire progressivement un langage :

```text
dynamique
     +
expressif
     +
prévisible
     +
testable
     +
extensible
```

---

# 📜 Licence

Kastel est distribué sous licence **MIT**.

Voir le fichier [`LICENSE`](LICENSE) pour les conditions complètes.

---

# 👤 Auteur

**MAHASOLO Jean Bruno**

Projet personnel de conception d'un langage de programmation dynamique et de sa machine virtuelle en Rust.

Repository :

https://github.com/MJBruno/Kastel
