# CONCURENCE

## 1. Concurrence coopérative

La première étape implémentée est une concurrence coopérative, avec :

```Kastel
spawn(function, args...)
yield()
task.join()
task.status()
task.is_done()
Exemple
let log = [];

func worker(id: int) -> int {
    log.add(id);

    yield();

    log.add(id + 10);

    return id;
}

let first = spawn(worker, 1);
let second = spawn(worker, 2);

let a = first.join();
let b = second.join();

println(a);
println(b);
println(log);
println(first.status());
println(first.is_done());

```

Résultat attendu :

```text
1
2
[1, 2, 11, 12]
done
true
```

Le scheduler fait :

```text
Task 1
  ↓
log.add(1)
  ↓
yield()
  ↓
Task 2
  ↓
log.add(2)
  ↓
yield()
  ↓
Task 1
  ↓
log.add(11)
  ↓
completed
  ↓
Task 2
  ↓
log.add(12)
  ↓
completed
Préemption automatique
```

yield() n'est pas obligatoire.

Le scheduler utilise également un quantum de 1024 instructions :

```text
Task A
  ↓ 1024 instructions
pause
  ↓
Task B
  ↓ 1024 instructions
pause
  ↓
Task A
```

Cela évite qu'une tâche purement CPU bloque complètement les autres tâches.

### Architecture

La structure est maintenant :

```text
VM principale
│
├── Scheduler
│    │
│    ├── Task 0
│    │    └── VirtualMachine
│    │
│    ├── Task 1
│    │    └── VirtualMachine
│    │
│    └── Task 2
│         └── VirtualMachine
│
└── Globals partagées
```

Chaque tâche possède sa propre :

stack
frames
exception_handlers
pending_exception
open_upvalues
temp_roots

mais les tâches partagent :

globals
module loader

Cela évite surtout de partager une même pile VM entre plusieurs contextes d'exécution.

### GC

Le GC connaît maintenant les tâches en attente :

```text
GC roots
├── VM principale
├── stack
├── globals
├── frames
├── open upvalues
└── Scheduler
    ├── Task 0
    │   ├── stack
    │   ├── frames
    │   ├── globals
    │   └── result
    └── Task 1
        └── ...
```

Le résultat terminé d'une tâche est également enraciné afin qu'un objet retourné par :

```Kastel
let value = task.join();
```

ne puisse pas être détruit prématurément par le GC.

Sécurité des closures

Pour cette première version, ceci est volontairement refusé :

```Kastel
func make_task() {
    let x = 10;

    let task = spawn(() => x + 1);
}
```

La raison est importante : x est une variable locale portée par une frame de la VM appelante. La déplacer telle quelle dans une autre VM demanderait un modèle explicite de capture/move/shared state.

Kastel produit donc :

```Kastel
TaskCaptureNotAllowed
```

En revanche, ceci est valide :

```Kastel
func worker() -> int {
    return 42;
}

let task = spawn(worker);

```

ainsi que :

```Kastel
func worker(value: int) -> int {
    return value * 2;
}

let task = spawn(worker, 21);
println(task.join());
```

Les fonctions surchargées et les méthodes liées sont également prises en charge par spawn.

Erreurs

Une erreur d'une tâche est conservée par le scheduler et ressort de join() :

```Kastel
try {
    let value = task.join();
}
catch (e: Err) {
    println(e.kind);
    println(e.message);
}
```

Donc la séparation reste :

```text
Result<T,E>
    ↓
erreur contrôlée
    ↓
    ?
Exception
    ↓
try / catch

Task
    ↓
join()
    ↓
valeur ou RuntimeError
```
Ce qui est volontairement absent de cette V1

Il n'y a pas encore :

```Kastel
threads OS
parallélisme CPU réel
Channel<T>
Mutex<T>
Atomic<T>
select
async/await
cancel()
```
