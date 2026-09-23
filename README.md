[![Rust](https://img.shields.io/badge/Rust-2024-orange?style=for-the-badge\&logo=rust)](https://www.rust-lang.org/)
[![License-MIT](https://img.shields.io/badge/License-MIT-blue?style=for-the-badge)](LICENSE)
[![Bytecode-VM](https://img.shields.io/badge/Bytecode-VM-brightgreen?style=for-the-badge)](src/vm/)
[![Status](https://img.shields.io/badge/Status-In%20Development-yellow?style=for-the-badge)](#-état-du-projet)

# 🏰 Kastel

> Un langage de programmation dynamique, compilé en bytecode et exécuté par une machine virtuelle stack-based écrite en Rust.

**Kastel** est un langage de programmation dynamique conçu autour d'une architecture d'exécution explicite :

```text
Source Kastel
     │
     ▼
   Lexer
     │
     ▼
   Parser
     │
     ▼
     AST
     │
     ▼
  Compiler
     │
     ▼
  Bytecode
     │
     ▼
 Stack-Based VM
     │
     ├──────────► Runtime
     │
     ├──────────► Native Library
     │
     └──────────► Garbage Collector
```

Le projet cherche à combiner :

```text
syntaxe dynamique
        +
expressivité
        +
bytecode
        +
VM explicite
        +
runtime structuré
        +
gestion mémoire automatique
```

L'objectif n'est pas uniquement de créer un langage avec une syntaxe moderne, mais de construire **chaque couche du runtime de manière compréhensible, testable et extensible**.

> Kastel est actuellement en phase de consolidation : les fonctionnalités principales sont en place et le travail porte désormais en priorité sur la robustesse, les tests, les diagnostics, les performances et la cohérence du runtime.

---

# ✨ Fonctionnalités

| Domaine       | Fonctionnalités                                               |
| ------------- | ------------------------------------------------------------- |
| Langage       | Typage dynamique, annotations de types optionnelles, inférence et types union                                              |
| Variables     | `let`, `const`                                                |
| Fonctions     | Fonctions, récursivité, fonctions imbriquées, fonctions anonymes, fonctions fléchées                  |
| Closures      | Captures lexicales, upvalues, closures                        |
| Collections   | Listes dynamiques, dictionnaires, objets                    |
| Contrôle      | `if`,`match`, boucles, `break`, `continue`                            |
| Itération     | `for .. in`, itérateurs, `range()` lazy                       |
| OOP           | Classes, instances, champs, méthodes                          |
| Héritage      | Héritage simple, override, `base`                             |
| Constructeurs | `new`, `initialize`, constructeurs hérités                          |
| Interfaces    | Interfaces multiples, héritage d'interfaces, validation       |
| Méthodes      | `this`, méthodes liées (`BoundMethod`)                        |
| Modules       | `import`, `export`, chargement de modules                     |
| Bytecode      | Chunks, constantes, opcodes, désassemblage                    |
| Runtime       | VM stack-based                                                |
| Mémoire       | Garbage Collector mark/sweep avec gestion des cycles          |
| Native        | I/O, math, strings, lists, objects, iterators, system, debug |
| Diagnostics   | Erreurs de compilation et runtime localisées                  |
| Debug         | VM trace, GC trace, désassemblage bytecode                    |

---

# 🧠 Architecture

Kastel est organisé en plusieurs couches indépendantes.

```text
                         Source Kastel
                              │
                              ▼
                         ┌─────────┐
                         │  Lexer  │
                         └────┬────┘
                              │
                              ▼
                         ┌─────────┐
                         │ Parser  │
                         └────┬────┘
                              │
                              ▼
                         ┌─────────┐
                         │   AST   │
                         └────┬────┘
                              │
                              ▼
                         ┌─────────┐
                         │Compiler │
                         └────┬────┘
                              │
                              ▼
                         ┌─────────┐
                         │ Bytecode│
                         └────┬────┘
                              │
                              ▼
                       ┌──────────────┐
                       │ Stack-Based  │
                       │      VM      │
                       └──────┬───────┘
                              │
              ┌───────────────┼────────────────┐
              ▼               ▼                ▼
          Runtime           Native          Modules
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
    → analyse du programme et génération du bytecode

bytecode
    → opcodes, chunks, constantes et désassemblage

vm
    → exécution du bytecode

runtime
    → Value, Object, fonctions, closures, upvalues et GC

native
    → bibliothèque standard native

module
    → résolution, chargement et gestion des modules

error
    → erreurs de compilation et erreurs runtime
```

Cette séparation permet de faire évoluer le langage sans concentrer toute la logique dans la VM.

---

# ⚙️ Machine virtuelle

Kastel utilise une **machine virtuelle à pile**.

Les appels de fonctions sont représentés par des `CallFrame` :

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
├── callees
├── arguments
├── locals
├── receivers
└── temporary values
```

Le fonctionnement général est :

```text
Bytecode
   │
   ▼
Fetch instruction
   │
   ▼
Decode opcode
   │
   ▼
Dispatch
   │
   ▼
Runtime operation
   │
   ▼
Stack / Heap / Frame
```

La VM protège notamment :

```text
stack access
constant access
local slots
upvalue slots
jump targets
func calls
argument counts
opcode validity
```

Lorsqu'une opération invalide provient du programme ou du bytecode, le runtime doit produire un `RuntimeError` plutôt qu'un `panic!` Rust.

---

# 🔢 Valeurs runtime

Kastel utilise une représentation dynamique des valeurs.

Les valeurs primitives comprennent notamment :

```text
Integer
Float
Boolean
Nil
NativeFunction
Range
```

Les objets alloués sur le tas passent par une représentation commune :

```text
Value::Object
        │
        ▼
     Gc<Object>
```

Le système d'objets peut représenter :

```text
String
List
Dict
Function
Closure
BoundMethod
Iterator
Module
Class
Instance
Interface
```

Cette architecture permet au Garbage Collector de parcourir une structure d'objets unifiée.

---

# 🔗 Fonctions, Closures et Upvalues

Kastel supporte les **fonctions imbriquées**, les **closures** et les **captures lexicales**.

Une variable capturée peut évoluer selon le modèle :

```text
Local
  │
  ▼
Upvalue
```

et une capture peut elle-même être relayée :

```text
Local
  │
  ▼
Upvalue
  │
  ▼
Upvalue
```

Une upvalue ouverte référence initialement un slot de la stack :

```text
Closure
   │
   ▼
Upvalue
   │
   ▼
Stack slot
```

Lorsque le scope disparaît, la valeur peut être fermée :

```text
Closure
   │
   ▼
Upvalue
   │
   ▼
closed Value
```

Exemple :

```kastel
func make_counter() {
    let count = 0;

    func increment() {
        count = count + 1;
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

```kastel
1
2
3
```

---

# 🧩 Typage

Kastel utilise un modèle de typage dynamique avec des **annotations de types optionnelles** et de l'**inférence**.

Une annotation peut être ajoutée explicitement :

```kastel
let name: str = "Bruno";
let count: int = 10;
```

L'inférence permet de conserver une écriture concise :

```kastel
let count = 10;
let message = "hello";
```

Les types union permettent d'accepter plusieurs types :

```kastel
let value: int|float = 10;

func multiply_by_four(value: int|float) -> float {
    return value * 4;
}
```

Les collections peuvent également être annotées :

```kastel
let values: List<int|float> = [];
```

Ce modèle permet d'utiliser les types progressivement sans abandonner le caractère dynamique du langage.

---

# 🏛️ Programmation orientée objet

Kastel dispose d'un système OOP basé sur :

```text
Class
Instance
Method
Field
Inheritance
Interface
BoundMethod
```

## Classes

```kastel
class Person {
    func initialize(name) {
        this.name = name;
    }

    func greet() {
        println(this.name);
    }
}

let person = new Person("Bruno");

person.greet();
```

## Champs d'instance

Chaque instance possède ses propres champs :

```kastel
class User {
    func show() {
        println(this.name);
    }
}

let a = new User();
let b = new User();

a.name = "Alice";
b.name = "Bob";

a.show();
b.show();
```

Résultat :

```text
Alice
Bob
```

Les champs sont stockés séparément pour chaque instance.

---

## Héritage

Kastel supporte l'héritage simple :

```kastel
class Animal {
    func speak() {
        println("animal");
    }
}

class Dog : Animal {
    func speak() {
        println("dog");
    }
}

let dog = new Dog();

dog.speak();
```

Résultat :

```text
dog
```

---

## `base`

Une méthode peut appeler explicitement l'implémentation du parent :

```kastel
class Animal {
    func speak() {
        println("animal");
    }
}

class Dog : Animal {
    func speak() {
        println("dog");
        base.speak();
    }
}

let dog = new Dog();

dog.speak();
```

Résultat :

```text
dog
animal
```

La résolution de `base` fonctionne également avec plusieurs niveaux d'héritage.

---

## `initialize` et constructeurs

Les instances sont créées avec `new` :

```kastel
class Person {
    func initialize(name) {
        this.name = name;
    }
}

let person = new Person("Bruno");
```

Les constructeurs peuvent être hérités :

```kastel
class Animal {
    func initialize(name) {
        this.name = name;
    }
}

class Dog : Animal {}

let dog = new Dog("Rex");
```

Une classe enfant peut appeler explicitement son constructeur parent :

```kastel
class Animal {
    func initialize(name) {
        this.name = name;
    }
}

class Dog : Animal {
    func initialize(name, age) {
        base.initialize(name);
        this.age = age;
    }
}

let dog = new Dog("Rex", 5);
```

La valeur retournée par `initialize` ne remplace pas l'instance créée par `new`.

---

## Méthodes comme valeurs

Les méthodes peuvent être récupérées comme valeurs :

```kastel
class Counter {
    func show() {
        println(this.value);
    }
}

let counter = new Counter();

counter.value = 42;

let show = counter.show;

show();
```

Kastel crée alors une méthode liée :

```text
BoundMethod
├── method
└── receiver
```

Le `receiver` reste associé à la méthode :

```kastel
let show = counter.show;

counter = null;

show();
```

La méthode conserve son instance.

---

## Résolution des propriétés

La résolution d'une propriété d'instance suit cette logique :

```text
instance.property
       │
       ▼
champ d'instance ?
   │          │
  oui        non
   │          │
   ▼          ▼
 Value     recherche
           dans la hiérarchie
                │
                ▼
           BoundMethod
```

Ainsi un champ peut masquer une méthode :

```kastel
func field_callback() {
    println("field");
}

class Test {
    func callback() {
        println("method");
    }

    func initialize() {
        this.callback = field_callback;
    }
}

let test = new Test();

test.callback();
```

Résultat :

```code
field
```

---

# 🔌 Interfaces

Kastel supporte les interfaces et leur héritage.

```kastel
interface Printable {
    func print();
}

class Document : Printable {
    func print() {
        println("document");
    }
}
```

Une classe peut implémenter plusieurs interfaces :

```kastel
interface Printable {
    func print();
}

interface Serializable {
    func save();
}

class Document : Printable, Serializable {
    func print() {
        println("print");
    }

    func save() {
        println("save");
    }
}
```

Les interfaces peuvent également hériter d'autres interfaces :

```kastel
interface Printable {
    func print();
}

interface Document : Printable {
    func save();
}
```

Le runtime vérifie les contrats d'interface :

```text
méthode manquante
        ↓
InterfaceMethodMissing

mauvaise arité
        ↓
InterfaceMethodArityMismatch

contrats incompatibles
        ↓
RuntimeError
```

---

# `is`

Le langage fournit l'opérateur `is` pour tester l'appartenance d'une instance à une classe ou une interface :

```kastel
class Animal {}

class Dog : Animal {}

let dog = new Dog();

println(dog is Dog);
println(dog is Animal);
```

Résultat :

```kastel
true
true
```

Les interfaces sont également supportées :

```kastel
interface Printable {
    func print();
}

class Document : Printable {
    func print() {
        println("document");
    }
}

let document = new Document();

println(document is Printable);
```

Résultat :

```kastel
true
```

---

# 🧹 Garbage Collector

Kastel utilise actuellement un Garbage Collector basé sur :

```text
MARK
  ↓
SWEEP
```

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

Les racines peuvent notamment provenir de :

```text
VM stack
globals
call frames
open upvalues
modules
func constants
closed upvalues
```

Les relations entre objets sont parcourues récursivement :

```text
Class
├── superclass
├── interfaces
└── methods

Instance
├── class
└── fields

Closure
├── func
├── upvalues
└── owner_class

BoundMethod
├── method
└── receiver

Interface
└── bases
```

## Gestion des cycles

Le GC doit pouvoir récupérer des graphes cycliques qui ne sont plus atteignables depuis les racines.

Exemple :

```text
Instance
   │
   ├──────► self
   │
   └──────► BoundMethod
                 │
                 └──────► receiver
```

Le système actuel détecte et casse les cycles inaccessibles pendant le sweep.

---

# 📦 Bibliothèque standard

La bibliothèque native est organisée par domaines.

```text
src/native/
├── mod.rs
├── io.rs
├── math.rs
├── string.rs
├── list.rs
├── object.rs
├── iterator.rs
├── system.rs
└── debug.rs
```

## I/O

```kastel
print(...)
println(...)
input(...)
```

## Math

```kastel
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

## Strings

```kastel
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

## Lists

```kastel
list.add(value)
list.remove(index)
list.size()

list.insert(index, value)
list.contains(value)
list.clear()
```

## Objects

```kastel
object()

object.get(key)
object.set(key, value)

object.has(key)
object.keys()
object.values()
object.length()
```

## Iterateurs

```kastel
range(...)
list(iterator)
```

`range()` est conçu comme une représentation légère et lazy.

```kastel
for i in range(5) {
    println(i);
}
```

Résultat :

```kastel
0
1
2
3
4
```

## System

```kastel
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

```kastel
inspect(value)
debug(value)
```

---

# 🔁 Itération

Kastel utilise une syntaxe `for .. in` :

```kastel
for value in [10, 20, 30] {
    println(value);
}
```

Avec une plage :

```kastel
for i in range(5) {
    println(i);
}
```

Plage avec début, fin et pas :

```kastel
for i in range(2, 10, 2) {
    println(i);
}
```

Résultat :

```kastel
2
4
6
8
```

Les plages descendantes sont également supportées :

```kastel
for i in range(10, 0, -1) {
    println(i);
}
```

Les tableaux, dictionnaires, chaînes, plages et itérateurs peuvent participer au système d'itération selon leur support runtime.

---

# 📦 Modules

Kastel possède un système de modules permettant de séparer un programme en plusieurs fichiers.

Les opérations principales sont :

```kastel
import dog.Dog;
from dog import *;
import dog { Dog, Animals, details };
```

Les symboles publics peuvent être exposés avec `export`.

Le runtime utilise un `ModuleLoader`.

Les objectifs du système sont notamment :

```text
module resolution
module loading
module caching
exports
runtime isolation
module errors
```

Le système de modules fait partie des composants actuellement consolidés avec le reste du runtime.

---

# 🛡️ Gestion des erreurs

Kastel distingue :

```text
CompileError
     +
RuntimeError
```

Les diagnostics peuvent conserver :

```text
ligne
colonne
message
```

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

Le principe général est :

```text
Programme invalide
       │
       ▼
  CompileError
       │
       ▼
   Diagnostic
```

ou :

```text
Programme valide
       │
       ▼
Exécution invalide
       │
       ▼
 RuntimeError
       │
       ▼
   Diagnostic
```

Les erreurs internes liées à un programme Kastel ne doivent pas nécessiter un `panic!` Rust.

---

# 🧪 Tests

Le développement de Kastel suit une approche orientée régression :

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

Les différentes couches sont testées progressivement :

```text
Lexer
Parser
Compiler
Bytecode
VM
Functions
Closures
Upvalues
Lists
Objects
Iterators
Modules
Classes
Inheritance
Interfaces
GC
Runtime Errors
Standard Library
```

Commandes principales :

```bash
cargo check
cargo test
```

---

# 🔍 Debug et instrumentation

Kastel possède plusieurs mécanismes de diagnostic pour observer le fonctionnement interne du runtime.

## Exécution normale

```bash
cargo run --bin kastel -- examples/main.ks
```

## Trace du Garbage Collector

```bash
cargo run --bin kastel --features trace_gc -q -- examples/main.ks
```

## VM trace

```bash
cargo run --bin kastel --features debug_trace -- examples/main.ks
```

## VM + GC

```bash
cargo run --bin kastel --features "debug_trace,trace_gc" -- examples/main.ks
```

Les traces permettent notamment d'observer :

```text
instructions
bytecode
stack
calls
allocations
GC marking
GC sweeping
cycles cassés
```

---

# 📁 Structure du projet

```text
Kastel/
│
├── examples/
│   └── *.ks
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
│   │   ├── list.rs
│   │   ├── object.rs
│   │   ├── iterator.rs
│   │   ├── system.rs
│   │   └── debug.rs
│   │
│   ├── runtime/
│   │   ├── object.rs
│   │   ├── value.rs
│   │   ├── function.rs
│   │   ├── gc.rs
│   │   └── ...
│   │
│   └── vm/
│
├── test/
├── Cargo.toml
├── Cargo.lock
├── LICENSE
└── README.md
```

---

# 🚧 État du projet

Kastel est actuellement en **phase de consolidation du runtime**.

Les mécanismes fondamentaux sont désormais en place :

```text
✅ Typage dynamique
✅ Lexer
✅ Parser
✅ AST
✅ Compiler
✅ Bytecode
✅ Stack-Based VM
✅ Variables locales et globales
✅ const
✅ Fonctions
✅ Récursivité
✅ Closures
✅ Upvalues
✅ Lists
✅ Dictionaries / Objects
✅ Iterators
✅ range()
✅ for .. in
✅ Modules
✅ Classes
✅ Instances
✅ Héritage
✅ Override
✅ this
✅ base
✅ initialize
✅ Constructeurs hérités
✅ Interfaces
✅ Héritage d'interfaces
✅ BoundMethod
✅ Opérateur is
✅ Garbage Collector
✅ Gestion des cycles
✅ Runtime errors
✅ Native functions
✅ VM / GC tracing
```

Le projet entre maintenant dans une phase où la priorité est davantage la **solidité** que l'accumulation rapide de fonctionnalités.

---

# 🎯 Priorités actuelles

```text
1.  Consolider le runtime
2.  Renforcer les tests de régression
3.  Finaliser et tester la bibliothèque standard
4.  Auditer le Compiler
5.  Auditer la VM
6.  Auditer le Garbage Collector
7.  Consolider le système de modules
8.  Uniformiser les diagnostics
9.  Réduire les duplications du runtime
10. Ajouter des benchmarks
11. Profiler les chemins critiques
12. Optimiser après mesure
13. Documenter la spécification du langage
```

La règle de développement est volontairement simple :

```text
Fonctionnalité
      ↓
Implémentation
      ↓
Tests
      ↓
Régression
      ↓
Profiling / mesure
      ↓
Optimisation
```

Les optimisations majeures, notamment un éventuel **JIT**, seront introduites après stabilisation et mesure du runtime existant.

---

# 🗺️ Évolution prévue

La trajectoire générale du projet est :

```text
                 Kastel
                    │
        ┌───────────┼───────────┐
        ▼           ▼           ▼
     Language     Runtime     Stdlib
        │           │           │
        ▼           ▼           ▼
     Syntax       VM / GC    Native API
        │           │           │
        └───────────┼───────────┘
                    ▼
             Tests / Benchmarks
                    │
                    ▼
               Profiling
                    │
                    ▼
             Stabilisation
                    │
                    ▼
              Optimisation
```

À plus long terme, Kastel pourra évoluer vers :

```text
Language
   │
   ├── richer standard library
   ├── stronger tooling
   ├── specification
   ├── formatter / diagnostics
   └── performance optimizations
                       │
                       ▼
                      JIT
```

Le principe reste de conserver une architecture où :

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

restent clairement séparés.

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

Compilation standard :

```bash
cargo build
```

Compilation optimisée :

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

# ▶️ Exécuter Kastel

Exécuter un programme :

```bash
cargo run --bin kastel -- demo/main.ks
```

Avec la trace GC :

```bash
cargo run --bin kastel --features trace_gc -q -- demo/main.ks
```

---

# 📜 Licence

Kastel est distribué sous licence **MIT**.

Voir [`LICENSE`](LICENSE) pour les conditions complètes.

---

# 👤 Auteur

**MAHASOLO Jean Bruno**

Projet personnel de conception d'un langage de programmation dynamique et de sa machine virtuelle en Rust.

Repository :

https://github.com/MJBruno/Kastel
