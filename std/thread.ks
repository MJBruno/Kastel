// Concurrence de haut niveau.
//
// Les appels directs `thread.spawn`, `thread.yield` et `thread.sleep` sont
// compilés comme les intrinsèques du VM. Ces fonctions exportées fournissent
// le contrat de module et gardent les mêmes noms disponibles dans le module.

export func spawn(task) {
    return spawn(task);
}

export func yield() {
    yield();
}

export func sleep(milliseconds: int) {
    sleep(milliseconds);
}
