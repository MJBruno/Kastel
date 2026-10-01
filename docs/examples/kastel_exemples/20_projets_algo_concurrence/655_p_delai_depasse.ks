// ==================================================================
// Exemple 655 — Délai dépassé
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : attendre un résultat au plus 20 ms avec select, puis annuler la tâche trop lente.
// ------------------------------------------------------------------
// Sortie attendue :
//   délai dépassé
//   TaskCancelled
// ==================================================================

let resultat = channel<int>();

func lent() {
    sleep(500);
    resultat.send(1);
}

let t = spawn(lent);
let r = select([resultat], 20);
if r[0] == -1 {
    println("délai dépassé");
    t.cancel();
}

try {
    t.join();
} catch (e: Err) {
    println(e.kind);
}
