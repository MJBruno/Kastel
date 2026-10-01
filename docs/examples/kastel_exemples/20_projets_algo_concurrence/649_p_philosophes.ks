// ==================================================================
// Exemple 649 — Dîner des philosophes
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : chacun prend toujours sa plus petite fourchette en premier, ce qui évite l'interblocage.
// ------------------------------------------------------------------
// Sortie attendue :
//   6
// ==================================================================

let fourchettes = [mutex(), mutex(), mutex()];
let repas = 0;

func philosophe(id: int) {
    let a = min(id, (id + 1) % 3);
    let b = max(id, (id + 1) % 3);
    for i in range(2) {
        fourchettes[a].lock();
        fourchettes[b].lock();
        repas += 1;
        fourchettes[b].unlock();
        fourchettes[a].unlock();
        yield();
    }
}

let taches = [];
for i in range(3) {
    taches.add(spawn(philosophe, i));
}
for t in taches {
    t.join();
}
println(repas);
