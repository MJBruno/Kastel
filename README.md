<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.70%2B-orange?style=for-the-badge&logo=rust" alt="Rust Version">
  <img src="https://img.shields.io/badge/Status-En%20Développement-yellow?style=for-the-badge" alt="Status">
  <img src="https://img.shields.io/badge/Bytecode-VM-brightgreen?style=for-the-badge" alt="Bytecode VM">
</p>
[<img src="https://img.shields.io/badge/License-MIT-blue?style=for-the-badge" alt="License MIT">](LICENSE)
# 🏰 Kastel

> Un langage de programmation dynamique, interprété par une machine virtuelle et compilé en bytecode.

Kastel est un langage de programmation moderne écrit en **Rust**, conçu autour d'une architecture **compilateur → bytecode → VM stack-based**.

Le projet met l'accent sur une architecture de runtime claire, les **closures**, les **upvalues**, la gestion automatique de la mémoire et un système d'erreurs permettant de diagnostiquer les problèmes d'exécution.

Kastel est actuellement en phase de **stabilisation du runtime** : l'objectif n'est pas seulement d'ajouter des fonctionnalités, mais de rendre le compilateur, le bytecode, la VM et le Garbage Collector fiables et cohérents.

---

## ✨ Fonctionnalités

| Domaine | Fonctionnalités |
|---|---|
| Langage | Typage dynamique |
| Variables | `let`, `const` |
| Fonctions | Fonctions, récursivité, fonctions imbriquées |
| Closures | Closures, captures lexicales, upvalues |
| Collections | Tableaux dynamiques, objets |
| Contrôle | `if`, boucles, `for .. in`, `break`, `continue` |
| Itération | Itérateurs et `range()` |
| Opérateurs | Arithmétiques, comparaisons, bitwise |
| Modules | `import` / `export` |
| Runtime | VM stack-based |
| Compilation | Compilation en bytecode |
| Mémoire | Garbage Collector avec traçage des objets et upvalues |
| Debug | Désassembleur bytecode, traçage VM/GC |
| Erreurs | Erreurs de compilation et erreurs runtime avec localisation source |

---

## 🧠 Architecture

Kastel suit une architecture en plusieurs couches :

```text
             Source Kastel
                   │
                   ▼
              Lexer / Parser
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
          ┌────────┼────────┐
          ▼        ▼        ▼
       Runtime   Objects    Native
          │
          ▼
          GC
```

### Organisation principale

```text
src/
├── bytecode/
├── compiler/
├── error/
├── frontend/
├── module/
├── runtime/
└── vm/
```

Les responsabilités sont volontairement séparées :

```text
frontend
    → analyse syntaxique

compiler
    → génération du bytecode

bytecode
    → instructions, constantes, désassemblage

vm
    → exécution du bytecode

runtime
    → valeurs, objets, fonctions, closures, upvalues, GC

module
    → chargement des modules

error
    → diagnostics de compilation et d'exécution
```

---

# ⚙️ Machine virtuelle

Kastel utilise une **machine virtuelle basée sur une pile**.

Chaque appel de fonction possède un `CallFrame` :

```text
CallFrame
├── closure
├── instruction pointer (ip)
└── slot_start
```

Les appels sont organisés autour de la pile :

```text
                    Stack
                      │
          ┌───────────┼───────────┐
          ▼           ▼           ▼
       callee        args       locals
                                  │
                                  ▼
                              CallFrame
```

La VM protège explicitement ses opérations internes contre les états invalides :

```text
stack underflow
invalid constant
invalid local slot
invalid jump
invalid function
invalid opcode
wrong argument count
```

L'objectif est qu'un bytecode invalide produise une **`RuntimeError`** plutôt qu'un `panic!` Rust.

---

# 🔗 Closures et Upvalues

Les closures utilisent deux niveaux de représentation.

### Métadonnée de compilation

```rust
pub struct Upvalue {
    pub index: u8,
    pub is_local: bool,
}
```

Elle décrit comment une closure capture une variable.

### État runtime

```rust
pub struct ObjUpvalue {
    pub slot: usize,
    pub closed: Option<Value>,
}
```

Une upvalue peut être :

```text
OPEN

Closure
   │
   ▼
Upvalue
   │
   ▼
Stack slot
```

puis, lorsque le frame disparaît :

```text
CLOSED

Closure
   │
   ▼
Upvalue
   │
   ▼
closed Value
```

Cette séparation permet à une closure de continuer à utiliser une variable locale après la destruction du frame qui la contenait.

---

# 🧹 Garbage Collector

Kastel possède un Garbage Collector basé sur un système de **marquage et balayage des références cycliques**.

Le cycle général est :

```text
Roots
  │
  ▼
MARK
  │
  ▼
Reachable objects
  │
  ▼
SWEEP
  │
  ▼
Cycle breaking
```

Les racines principales comprennent notamment :

```text
Stack
Globals
CallFrames
OpenUpvalues
```

Le GC suit les relations entre :

```text
Object
Array
Dict
Function
Closure
Iterator
Module
Upvalue
```

Les `Function` parcourent également leurs constantes afin que les objets référencés par le bytecode restent atteignables :

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

Les upvalues fermées sont également traversées :

```text
ObjUpvalue
   │
   ▼
closed Value
```

---

# 🛡️ Robustesse du runtime

La stabilisation actuelle du runtime suit une règle simple :

```text
Programme Kastel invalide
        ↓
RuntimeError
        ↓
diagnostic
        ↓
aucun panic lié au bytecode
```

Les opérations sensibles utilisent des vérifications explicites :

```text
checked_add()
checked_sub()
.get()
.get_mut()
Result<T, RuntimeError>
```

Les erreurs d'exécution peuvent également conserver leur position dans le code source :

```text
ligne X, colonne Y : message
```

---

# 📦 Modules

Kastel possède un système de modules avec :

```kastel
import ...
export ...
```

Les modules sont chargés par le runtime via le `ModuleLoader`.

Chaque module possède son propre contexte d'exécution et peut exposer des valeurs au module appelant.

---

# 🔢 Exemple

```kastel
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

```text
1
2
3
```

---

# 🔁 Itération

Kastel utilise une syntaxe d'itération basée sur `for .. in`.

```kastel
for value in [10, 20, 30] {
    println(value);
}
```

Avec `range()` :

```kastel
for i in range(5) {
    println(i);
}
```

Résultat :

```text
0
1
2
3
4
```

---

# 🧪 Tests

Le projet possède une suite de tests permettant de vérifier progressivement :

```text
lexer
parser
compiler
bytecode
VM
functions
closures
upvalues
arrays
objects
iterators
modules
GC
runtime errors
```

Exécution :

```bash
cargo check
cargo test
```

Le principe de développement du projet est :

```text
Bug
 ↓
Correction
 ↓
Test de régression
 ↓
Fonctionnalité stabilisée
```

---

# 🔍 Debug VM

Le bytecode peut être inspecté avec le désassembleur.

Pour activer le traçage de la VM :

```bash
cargo run --features debug_trace -- examples/main.ks
```

Pour suivre le Garbage Collector :

```bash
cargo run --features trace_gc -- examples/main.ks
```

Les deux peuvent être activés simultanément :

```bash
cargo run --features "debug_trace,trace_gc" -- examples/main.ks
```

---

# 🚧 État du projet

Kastel est actuellement en développement actif.

La priorité actuelle est la **solidification du runtime** :

```text
Compiler
   ↓
Bytecode
   ↓
VM
   ↓
Closures / Upvalues
   ↓
GC
   ↓
Tests de régression
```

Le projet privilégie maintenant la stabilité des mécanismes existants avant l'ajout de fonctionnalités majeures.

### Priorités actuelles

```text
✅ Sécurisation de la pile VM
✅ Validation des CallFrames
✅ Validation des accès bytecode
✅ Validation des appels
✅ Validation des captures d'upvalues
✅ Séparation runtime / VM pour les upvalues
✅ Marquage des constantes de Function
⬜ Audit approfondi du Garbage Collector
⬜ Tests GC spécialisés
⬜ Renforcement des invariants runtime
⬜ Extension progressive de la bibliothèque standard
```

---

# 🛠️ Compiler Kastel

Prérequis :

- Rust
- Cargo

Cloner le projet :

```bash
git clone https://github.com/MJBruno/Kastel.git
cd Kastel
```

Compiler :

```bash
cargo build
```

Compiler en release :

```bash
cargo build --release
```

Exécuter un programme :

```bash
cargo run -- examples/main.ks
```

---

# 📁 Structure du projet

```text
Kastel/
├── examples/
├── showcase/
├── src/
│   ├── bytecode/
│   ├── compiler/
│   ├── error/
│   ├── frontend/
│   ├── module/
│   ├── runtime/
│   └── vm/
├── test/
├── Cargo.toml
├── Cargo.lock
└── README.md
```

---

# 🎯 Vision

Kastel est construit avec une idée centrale :

> **Un langage simple en surface, mais un runtime conçu avec des mécanismes solides.**

Le projet cherche à conserver une architecture compréhensible tout en permettant son évolution vers :

```text
Kastel
  │
  ├── langage dynamique
  ├── bytecode
  ├── VM
  ├── closures
  ├── upvalues
  ├── GC
  ├── modules
  └── runtime extensible
```

L'objectif n'est pas uniquement de faire fonctionner Kastel, mais de construire progressivement un **runtime fiable, testable et extensible**.

---

# 📜 Licence

Voir le fichier `LICENSE` du projet pour les conditions d'utilisation.

---

# 👤 Auteur

**MAHASOLO Jean Bruno**

Projet personnel de conception d'un langage de programmation et de sa machine virtuelle en Rust.

Repository :

https://github.com/MJBruno/Kastel