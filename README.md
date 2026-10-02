[![Rust](https://img.shields.io/badge/Rust-2024-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![License-MIT](https://img.shields.io/badge/License-MIT-blue?style=for-the-badge)](LICENSE)
[![Bytecode-VM](https://img.shields.io/badge/Bytecode-VM-brightgreen?style=for-the-badge)](src/vm/)
[![Status](https://img.shields.io/badge/Status-In%20Development-yellow?style=for-the-badge)](#-project-status)

# 🏰 Kastel

> A dynamic programming language compiled to bytecode and executed by a stack-based virtual machine written in Rust.

Kastel is designed around an explicit and modular runtime:

```text
Kastel source
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
     ├──────────────► Runtime
     │                  │
     │                  ├── Objects
     │                  ├── Closures
     │                  └── GC
     │
     ├──────────────► Scheduler
     │                  ├── Tasks
     │                  ├── Channels
     │                  └── Mutexes
     │
     └──────────────► Modules / Stdlib
```

The goal is to build each layer in a readable, testable, and extensible way without hiding the essential mechanisms of the virtual machine.

---

## ✨ Features

| Domain | Features |
| --- | --- |
| Language | Dynamic typing, optional annotations, inference, union types |
| Variables | `let`, `const`, local and global variables |
| Functions | Functions, recursion, nested functions, anonymous functions, arrow functions |
| Closures | Lexical captures and upvalues |
| Collections | `List`, dictionaries, and objects |
| Control flow | `if`, `match`, loops, `break`, `continue` |
| Iteration | `for ... in`, iterators, and lazy `range()` |
| OOP | Classes, instances, fields, and methods |
| Constructors | `new` and `initialize` |
| Interfaces | Interface contracts and class validation |
| Enums | Qualified variants, e.g. `Color.Red` |
| Modules | `import`, `from ... import`, `export`, and module loading |
| Exceptions | `try`, `catch`, and runtime error propagation |
| VM | Bytecode, chunks, constants, stack, and `CallFrame` |
| Memory | Mark/sweep garbage collection and cycle collection |
| Concurrency | Cooperative tasks, `spawn`, `join`, `yield`, `sleep`, cancellation |
| Synchronization | Channels, `select`, and mutexes |
| Diagnostics | Compilation/runtime errors with source locations |
| Debugging | VM trace, GC trace, and bytecode disassembly |

---

## 🧠 Architecture

Kastel separates responsibilities into several layers:

```text
                          Kastel source
                               │
                               ▼
                         ┌───────────┐
                         │   Lexer   │
                         └─────┬─────┘
                               │
                               ▼
                         ┌───────────┐
                         │  Parser   │
                         └─────┬─────┘
                               │
                               ▼
                         ┌───────────┐
                         │    AST    │
                         └─────┬─────┘
                               │
                               ▼
                         ┌───────────┐
                         │ Compiler  │
                         └─────┬─────┘
                               │
                               ▼
                         ┌───────────┐
                         │ Bytecode  │
                         └─────┬─────┘
                               │
                               ▼
                         ┌───────────┐
                         │    VM     │
                         └─────┬─────┘
                               │
              ┌────────────────┼────────────────┐
              ▼                ▼                ▼
           Runtime          Scheduler         Modules
              │                │
              ▼                ├── Tasks
              GC               ├── Channels
                               └── Mutexes
```

### Code organization

```text
src/
├── app/
├── bytecode/
├── compiler/
├── error/
├── frontend/
├── module/
├── runtime/
│   ├── channel.rs
│   ├── closure.rs
│   ├── function.rs
│   ├── gc.rs
│   ├── gc_handle.rs
│   ├── hashed.rs
│   ├── iterator.rs
│   ├── mutex.rs
│   ├── object.rs
│   ├── upvalue.rs
│   └── value.rs
├── stdlib/
└── vm/
    └── machine/
        ├── concurrency_tests.rs
        └── scheduler.rs
```

`stdlib` contains the standard library interface exposed to Kastel. Documentation should no longer reference `src/native/`.

---

# ⚙️ Virtual machine

Kastel uses a **stack-based VM**.

Each function call uses a `CallFrame` containing, among other fields:

```text
CallFrame
├── closure
├── instruction pointer (ip)
└── slot_start
```

The stack contains, among other things:

```text
Stack
├── callees
├── arguments
├── locals
├── receivers
└── temporary values
```

The general execution cycle is:

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

Critical VM accesses are validated so that an error originating in a Kastel program produces a `RuntimeError` rather than a Rust `panic!`.

---

# 🧩 Runtime values and objects

Kastel uses a dynamic value representation.

Primitive values include:

```text
Integer
Float
Boolean
Nil
NativeFunction
Range
```

Heap values use:

```text
Value::Object
       │
       ▼
    Gc<Object>
```

Runtime objects include:

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
Task
Channel
Mutex
```

This common representation allows the GC and the runtime to traverse a coherent object graph.

---

# 🔗 Functions, closures, and upvalues

Kastel supports nested functions and lexical captures.

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

```text
1
2
3
```

A closure can therefore retain a logical reference to a variable whose original scope has already disappeared.

---

# 🔤 Typing

Kastel remains dynamically typed while allowing optional annotations and inference.

```kastel
let name: str = "Bruno";
let count: int = 10;

let message = "hello";
```

Union types allow multiple possible types to be expressed:

```kastel
let value: int|float = 10;
```

Collections can also be annotated:

```kastel
let values: List<int|float> = [];
```

The model aims to retain the flexibility of a dynamic language without discarding type information when it is useful.

---

# 🏛️ Object-oriented programming

Kastel provides classes, instances, fields, and methods.

Classes do not use a class inheritance system. A class can instead satisfy the contract defined by one or more interfaces.

In instance methods, the reference to the current instance is `self`. Kastel does not use `this`.

### Classes

```kastel
class Person {
    func initialize(name) {
        self.name = name;
    }

    func greet() {
        println(self.name);
    }
}

let person = new Person("Bruno");
person.greet();
```

### Constructors

The conventional constructor is `initialize()`:

```kastel
class User {
    func initialize(name, age) {
        self.name = name;
        self.age = age;
    }
}

let user = new User("Bruno", 30);
```

### Interfaces

An interface defines a contract that classes can satisfy:

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

The class must provide the methods required by the interface contract. Interfaces can be combined according to the language type rules.

---

# 🔁 Iteration

Kastel uses `for ... in`:

```kastel
for value in [10, 20, 30] {
    println(value);
}
```

Ranges are represented by `range()`:

```kastel
for i in range(5) {
    println(i);
}

for i in range(2, 10, 2) {
    println(i);
}
```

Ranges are designed as lightweight, lazy values.

---

# 📦 Modules

Kastel has a module system with module loading, symbol resolution, and symbol exports.

```kastel
import dog.Dog;
from dog import *;
import dog { Dog, Animals, details };
```

Public symbols can be exposed with `export`.

```kastel
export class Dog : Printable {
    func initialize(name) {
        self.name = name;
    }

    func print() {
        println(self.name);
    }
}
```

The runtime uses a `ModuleLoader` for module resolution and loading.

---

# 🧵 Concurrency

Kastel now has a **cooperative concurrency** layer integrated into the VM.

The scheduler manages task states:

```text
Ready
Waiting
Completed
Failed
Cancelled
```

## Tasks

```kastel
func worker() {
    println("worker");
}

let task = spawn(worker);
task.join();
```

Main operations include:

```text
spawn(...)
join()
yield()
sleep(...)
task.status()
task.is_done()
task.cancel()
```

The scheduler exécute les tâches par quanta d'instructions et les suspend lorsqu'elles attendent une ressource.

## Channels

Channels allow values to be transferred between tasks:

```kastel
let ch = channel<int>();

ch.send(42);
let value = ch.recv();
```

Available operations include:

```text
send
recv
try_recv
close
select
```

The scheduler can wake waiting tasks and handle timeouts and channel closures.

## Mutexes

Mutexes provide cooperative synchronization at the Kastel task level.

The scheduler tracks the owner and a queue of blocked tasks, then wakes the next waiter when the mutex is released.

## Current model

The current concurrency model does not assign an independent OS thread to each Kastel task. It relies on a **cooperative scheduler integrated into the VM**.

```text
VM
 │
 ▼
Scheduler
 ├── Task A ──► Running
 ├── Task B ──► Ready
 ├── Task C ──► Waiting on Channel
 └── Task D ──► Waiting on Mutex
```

This architecture allows tasks, channels, mutexes, and their GC references to integrate directly with the existing runtime.

---

# 🧹 Garbage collector

Kastel uses a garbage collector based on:

```text
MARK
  ↓
SWEEP
```

Roots include:

```text
VM stack
globals
call frames
open upvalues
modules
function constants
closed upvalues
scheduler tasks
scheduler waiting values
```

The GC must also traverse references introduced by concurrency objects:

```text
Task
Channel
Mutex
```

Les graphes cycliques inaccessibles doivent rester collectables même lorsqu'ils contiennent des références mutuelles.

---

# 📚 Standard library

The standard library is organized under `src/stdlib/`.

It currently covers:

```text
I/O
Math
Strings
Lists
Objects
Iterators
System
Debug
Channels
Mutexes
```

Exemples d'API :

```kastel
print(...)
println(...)
input(...)

list.add(value)
list.remove(index)
list.size()
list.insert(index, value)
list.contains(value)
list.clear()

range(...)

clock()
cwd()
env(name)

inspect(value)
debug(value)
```

The standard library evolves with the needs of the language; detailed module documentation should remain aligned with the API actually exposed by `src/stdlib/`.

---

# 🛡️ Error handling

Kastel primarily distinguishes:

```text
CompileError
RuntimeError
```

Diagnostics can include:

```text
ligne
colonne
message
```

Possible runtime errors include:

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

Errors originating from a Kastel program should not require a Rust `panic!`.

---

# 🧪 Testing

The project follows a regression-driven approach:

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

Tests cover the main runtime and language layers, including:

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
Interfaces
Enums
Exceptions
GC
Runtime Errors
Standard Library
Concurrency
  ├── Tasks
  ├── Scheduling
  ├── Channels
  ├── Select
  ├── Mutexes
  └── Cancellation
```

Main commands:

```bash
cargo check
cargo test
```

---

# 🔍 Debugging and instrumentation

Kastel provides several tracing levels.

### VM trace

```bash
cargo run --bin kastel --features debug_trace -- demo/main.ks
```

### GC trace

```bash
cargo run --bin kastel --features trace_gc -- demo/main.ks
```

### VM + GC

```bash
cargo run --bin kastel --features "debug_trace,trace_gc" -- demo/main.ks
```

These tools make it possible to inspect:

```text
instructions
bytecode
stack
calls
allocations
GC marking
GC sweeping
scheduler activity
```

---

# 📁 Project structure

```text
Kastel/
│
├── demo/
│   └── main.ks
│
├── src/
│   ├── app/
│   ├── bytecode/
│   ├── compiler/
│   ├── error/
│   ├── frontend/
│   ├── module/
│   ├── runtime/
│   │   ├── channel.rs
│   │   ├── closure.rs
│   │   ├── function.rs
│   │   ├── gc.rs
│   │   ├── gc_handle.rs
│   │   ├── iterator.rs
│   │   ├── mutex.rs
│   │   ├── object.rs
│   │   ├── upvalue.rs
│   │   └── value.rs
│   ├── stdlib/
│   └── vm/
│       └── machine/
│           └── scheduler.rs
│
├── tests/
├── Cargo.toml
├── Cargo.lock
├── LICENSE
└── README.md
```

> The exact structure may evolve with the runtime. The README should reflect the modules actually present under `src/` rather than preserving a historical tree.

---

# 🚧 Project status

Kastel is currently in a **language and runtime consolidation** phase.

The current foundation includes:

```text
✅ Lexer / Parser / AST
✅ Compiler
✅ Bytecode
✅ Stack-Based VM
✅ Variables locales et globales
✅ Dynamic typing
✅ Annotations de types et inférence
✅ Fonctions / récursivité
✅ Closures / upvalues
✅ Lists / Dict / Objects
✅ Itérateurs / range / for-in
✅ Classes / instances
✅ initialize / constructeurs
✅ Interfaces et contrats
✅ Enums
✅ Modules
✅ Exceptions
✅ Garbage Collector
✅ Gestion des cycles
✅ Tasks coopératives
✅ Scheduler
✅ yield / sleep / join / cancellation
✅ Channels / select
✅ Mutexes
✅ Tests de régression
✅ Traces VM / GC
```

Current work prioritizes robustness and consistency across layers rather than adding features without consolidation.

---

# 🎯 Current priorities

```text
1.  Consolider le runtime
2.  Renforcer les tests de régression
3.  Finaliser et uniformiser la stdlib
4.  Auditer le compiler
5.  Auditer la VM
6.  Auditer le GC
7.  Consolider la concurrence
8.  Consolider le système de modules
9.  Uniformiser les diagnostics
10. Documenter la spécification du langage
11. Ajouter des benchmarks
12. Mesurer avant d'optimiser
```

The development principle remains:

```text
Fonctionnalité
      ↓
Implémentation
      ↓
Tests
      ↓
Régression
      ↓
Mesure / Profiling
      ↓
Optimisation
```

Any future JIT, AOT, or other major optimization strategy comes after the current runtime has been consolidated.

---

# 🗺️ Evolution

The fundamental separation of the project should remain clear:

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
   ├── GC
   └── Scheduler
        ├── Tasks
        ├── Channels
        └── Mutexes
```

Future work may include:

```text
spécification complète du langage
outillage / LSP / formatter
bibliothèque standard plus riche
benchmarks et profiling
optimisations VM
JIT / AOT à plus long terme
```

---

# 🛠️ Installation

## Prerequisites

- Rust
- Cargo

Kastel currently uses **Rust 2024**.

Clone the repository:

```bash
git clone https://github.com/MJBruno/Kastel.git
cd Kastel
```

---

# 🔨 Build

```bash
cargo build
```

Release build:

```bash
cargo build --release
```

Check:

```bash
cargo check
```

Tests:

```bash
cargo test
```

---

# ▶️ Running Kastel

Run a program:

```bash
cargo run --bin kastel -- demo/main.ks
```

Avec la trace GC :

```bash
cargo run --bin kastel --features trace_gc -- demo/main.ks
```

Avec la trace VM :

```bash
cargo run --bin kastel --features debug_trace -- demo/main.ks
```

---

# 📜 License

Kastel is distributed under the **MIT** license.

See [`LICENSE`](LICENSE) for the full license terms.

---

# 👤 Author

**MAHASOLO Jean Bruno**

Personal project focused on designing a dynamic programming language and its virtual machine in Rust.

Repository:

<https://github.com/MJBruno/Kastel>
