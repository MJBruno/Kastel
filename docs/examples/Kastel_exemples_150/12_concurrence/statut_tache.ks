// ====================================================================
// Kastel — Cycle de vie d'une tâche
// Notions : status() : ready, waiting, done
// Résultat attendu :
//   ready
//   waiting
//   done
// ====================================================================

func attendre(c) {
    return c.recv();
}

func scenario() {
    let c = channel();
    let t = spawn(attendre, c);

    println(t.status());     // créée, pas encore exécutée
    yield();
    println(t.status());     // bloquée sur recv
    c.send(1);
    t.join();
    println(t.status());
}

spawn(scenario).join();
