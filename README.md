![Architecture de Kastel](assets/logo/4.png)

# Kastel

**Kastel** est un langage de programmation moderne, dynamique et interprété, conçu pour offrir une syntaxe simple et expressive tout en reposant sur une architecture d'exécution basée sur une **machine virtuelle à bytecode**.

Inspiré de **JavaScript** et de **Python**, Kastel reprend certains de leurs principes — typage dynamique, syntaxe concise, fonctions, structures de données dynamiques et simplicité d'utilisation — tout en développant sa propre architecture interne.

Le langage et son runtime sont entièrement développés en **Rust**, sans dépendances externes actuellement.

## Fonctionnalités actuelles

Kastel dispose actuellement de plusieurs éléments fondamentaux d'un langage dynamique moderne :

- typage dynamique ;
- variables mutables et constantes ;
- fonctions ;
- appels de fonctions ;
- récursion ;
- closures ;
- upvalues ;
- tableaux dynamiques ;
- objets et propriétés ;
- indexation ;
- itérations ;
- fonctions natives ;
- modules et système d'import/export ;
- bytecode ;
- machine virtuelle stack-based ;
- garbage collector ;
- système d'erreurs séparé par couche ;
- désassembleur de bytecode.

Le projet continue d'évoluer et certaines fonctionnalités restent en cours de stabilisation et d'amélioration.

---

# Une syntaxe simple et expressive

Kastel cherche à conserver une syntaxe lisible avec une complexité syntaxique limitée.

Exemple :

```kastel
let name = "Kastel";
let version = 1;

function greet(name) {
    return "Hello, " + name;
}

println(greet(name));
```

La syntaxe est volontairement proche de langages comme JavaScript et Python afin de réduire la complexité nécessaire pour commencer à utiliser le langage.

---

# Typage dynamique

Kastel utilise un **système de typage dynamique**.

Les types sont déterminés au moment de l'exécution et les variables peuvent contenir différentes valeurs au cours de leur durée de vie.

```kastel
let value = 42;

value = "Kastel";

value = true;
```

Le runtime manipule notamment différentes catégories de valeurs :

```text
Value
├── Number
├── Boolean
├── String
├── Array
├── Object
├── Function
├── Closure
├── NativeFunction
├── Module
└── Nil
```

Cette approche donne au langage une grande flexibilité et convient particulièrement au scripting et au prototypage.

---

# Architecture d'exécution

Kastel utilise une architecture **source → bytecode → machine virtuelle**.

Le chemin principal est :

```text
                 Code Kastel
                      │
                      ▼
                   Lexer
                      │
                      ▼
                   Tokens
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
              Machine virtuelle
                      │
                      ▼
                  Runtime
```

Le compilateur transforme le programme Kastel en bytecode.

La machine virtuelle interprète ensuite ce bytecode et exécute les instructions.

Cette architecture permet de séparer clairement :

```text
Langage
   │
   ├── Frontend
   │
   ├── Compiler
   │
   ├── Bytecode
   │
   └── Runtime / VM
```

---

# Machine virtuelle

La VM de Kastel est une **machine virtuelle basée sur une pile**.

Son fonctionnement général repose sur :

```text
Bytecode
    │
    ▼
Instruction
    │
    ▼
Stack
    │
    ├── Values
    ├── Function calls
    ├── Local variables
    └── Temporary values
```

Les appels de fonctions utilisent des **Call Frames** afin de conserver l'état d'exécution de chaque fonction.

La VM gère également :

- les variables locales ;
- les variables globales ;
- les upvalues ;
- les closures ;
- les appels de fonctions ;
- les tableaux ;
- les objets ;
- les propriétés ;
- les itérateurs ;
- les modules ;
- les fonctions natives ;
- le garbage collector.

---

# Bytecode

Kastel possède son propre système de bytecode.

Le bytecode est organisé autour de :

```text
Chunk
├── code
└── constants
```

Les instructions sont représentées par des **opcodes**.

Exemples conceptuels :

```text
Constant
GetLocal
SetLocal
GetGlobal
SetGlobal

Add
Subtract
Multiply
Divide
Modulo

Jump
JumpIfFalse
Loop

Call
Return

Closure
GetUpvalue
SetUpvalue

Array
GetIndex
SetIndex

Object
GetProperty
SetProperty

Import
```

Le projet possède également un **désassembleur** permettant d'inspecter le bytecode généré par le compilateur.

---

# Fonctions, closures et upvalues

Kastel prend en charge les fonctions imbriquées et les closures.

Exemple :

```kastel
function counter() {
    let value = 0;

    function increment() {
        value = value + 1;
        return value;
    }

    return increment;
}
```

Une closure peut conserver l'accès aux variables de son environnement lexical grâce au système d'**upvalues**.

L'architecture correspondante est :

```text
Function
    │
    ▼
Closure
    │
    └── Upvalues
            │
            ▼
       Captured values
```

Cette partie constitue une composante importante du runtime de Kastel.

---

# Structures de données dynamiques

Kastel possède des structures de données dynamiques intégrées au runtime.

## Arrays

Les tableaux prennent en charge notamment :

```text
Array
├── get
├── set
├── length
├── push
├── pop
├── insert
├── remove
├── clear
└── contains
```

Exemple :

```kastel
let numbers = [10, 20, 30];

numbers.push(40);

println(numbers[0]);
println(numbers.length);
```

## Objects

Kastel possède également des objets dynamiques avec des propriétés :

```kastel
let user = {
    name: "Bruno",
    age: 25
};

println(user.name);
```

Les propriétés peuvent être lues et modifiées dynamiquement.

---

# Itération

Kastel possède un système d'itération permettant de parcourir les collections.

Exemple :

```kastel
for (value in [10, 20, 30]) {
    println(value);
}
```

Le runtime possède un protocole d'itération séparé de la logique générale de la VM.

Cette architecture permet d'étendre progressivement les types pouvant être parcourus.

---

# Fonctions natives

Le runtime fournit également des fonctions natives implémentées directement en Rust.

Elles permettent au langage d'accéder à des fonctionnalités qui ne sont pas nécessairement implémentées en bytecode Kastel.

Exemples :

```text
print
println
input
clock
range
int
float
str
bool
```

L'architecture est :

```text
Kastel
   │
   ▼
NativeFunction
   │
   ▼
Rust Runtime
```

Cela permet de créer progressivement une bibliothèque standard autour du langage.

---

# Modules

Kastel possède un système de modules permettant de séparer le code en plusieurs fichiers.

L'architecture comprend un **module loader** responsable du chargement des modules.

Le principe général est :

```text
Module A
   │
   │ import
   ▼
Module B
   │
   ▼
Module Loader
   │
   ▼
Compiler / VM
```

Le système d'import/export est actuellement en cours de développement et de stabilisation.

---

# Gestion de la mémoire

Le runtime utilise `Rc`, `RefCell` et `Weak` pour gérer différentes structures dynamiques.

Kastel possède également son propre mécanisme de garbage collection.

Le GC maintient notamment un registre d'objets et utilise les références faibles pour suivre les allocations du runtime.

Son fonctionnement général repose sur :

```text
VM Roots
   │
   ├── Stack
   ├── Globals
   ├── Call Frames
   └── Open Upvalues
          │
          ▼
        Mark
          │
          ▼
        Sweep
          │
          ▼
   Unreachable objects
```

Le garbage collector est encore une partie du runtime qui doit être progressivement renforcée et optimisée.

---

# Gestion des erreurs

Le système d'erreurs de Kastel est séparé en plusieurs catégories :

```text
error/
├── LexError
├── ParseError
├── CompileError
├── RuntimeError
├── MachineError
└── KastelError
```

Les différentes étapes du langage peuvent donc signaler des erreurs spécifiques :

```text
Source
  │
  ▼
Lexer ───────► LexError
  │
  ▼
Parser ──────► ParseError
  │
  ▼
Compiler ────► CompileError
  │
  ▼
VM ──────────► RuntimeError / MachineError
```

L'amélioration du système de diagnostics, notamment la localisation précise des erreurs dans le code source, fait partie des évolutions importantes du projet.

---

# Architecture du projet

La structure actuelle du projet est organisée autour des principaux composants du langage :

```text
src/
│
├── app/
│   └── Application
│
├── frontend/
│   ├── lexer.rs
│   ├── parser.rs
│   ├── token.rs
│   └── ast.rs
│
├── compiler/
│   ├── compiler.rs
│   ├── context.rs
│   ├── declarations.rs
│   ├── expressions.rs
│   ├── statements.rs
│   ├── functions.rs
│   ├── variables.rs
│   ├── locals.rs
│   ├── upvalue.rs
│   ├── scope.rs
│   ├── loops.rs
│   ├── control_flow.rs
│   └── emit.rs
│
├── bytecode/
│   ├── chunk.rs
│   ├── opcode.rs
│   └── disassembler.rs
│
├── vm/
│   ├── machine.rs
│   └── mod.rs
│
├── runtime/
│   ├── value.rs
│   ├── object.rs
│   ├── function.rs
│   ├── closure.rs
│   ├── upvalue.rs
│   ├── iterator.rs
│   ├── native.rs
│   └── gc.rs
│
├── module/
│   ├── module.rs
│   └── mod.rs
│
├── error/
│   ├── kastel_error.rs
│   ├── lex_error.rs
│   ├── parse_error.rs
│   ├── compile_error.rs
│   ├── runtime_error.rs
│   ├── machine_error.rs
│   └── mod.rs
│
└── main.rs
```

---

# Philosophie du projet

Kastel s'inspire de **JavaScript** et de **Python**, mais ne cherche pas à reproduire leur implémentation.

Le projet possède sa propre :

- syntaxe ;
- représentation AST ;
- architecture de compilation ;
- représentation bytecode ;
- machine virtuelle ;
- gestion des fonctions ;
- gestion des closures ;
- gestion des objets ;
- gestion des modules ;
- gestion de la mémoire ;
- architecture runtime.

L'objectif est de construire progressivement un langage cohérent plutôt que de simplement reproduire un langage existant.

---

# Objectifs du projet

Kastel poursuit plusieurs objectifs techniques :

- construire un langage dynamique complet ;
- concevoir un compilateur fonctionnel ;
- concevoir une machine virtuelle indépendante ;
- développer un runtime extensible ;
- expérimenter la gestion des closures et upvalues ;
- développer un système de modules ;
- améliorer la gestion de la mémoire ;
- construire un système d'erreurs et de diagnostics précis ;
- mesurer et améliorer les performances de la VM ;
- maintenir une architecture suffisamment modulaire pour permettre de futures optimisations.

---

# Évolution prévue

L'architecture actuelle constitue une base pour plusieurs évolutions futures.

```text
Kastel actuel
     │
     ├── Stabilisation du langage
     │
     ├── Tests complets
     │
     ├── Benchmarks
     │
     ├── Optimisation de la VM
     │
     ├── Amélioration du GC
     │
     ├── Diagnostics avancés
     │
     └── JIT éventuel
```

Le JIT n'est pas considéré comme une composante actuelle du langage : il représente une évolution possible de l'architecture d'exécution après la stabilisation et le profilage de la VM.

---

# Vision

La vision de Kastel est de construire progressivement un **langage dynamique moderne avec son propre environnement d'exécution**.

Le projet repose actuellement sur quatre piliers :

```text
        KASTEL
           │
   ┌───────┼────────┐
   ▼       ▼        ▼
Frontend Compiler  Runtime
           │        │
           ▼        ▼
        Bytecode    VM
```

L'objectif à long terme est d'obtenir une architecture où le langage, le bytecode, la machine virtuelle et le runtime sont suffisamment robustes pour permettre des optimisations avancées sans remettre en cause les fondations du projet.

**Kastel — un langage dynamique, son propre bytecode, sa propre machine virtuelle et son propre runtime.**