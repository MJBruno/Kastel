// ==================================================================
// Exemple 357 — Une liste de tâches
// Catégorie : async / await
// ------------------------------------------------------------------
// Lancer N tâches, puis les attendre dans une boucle.
// ------------------------------------------------------------------
// Sortie attendue :
//   55
// ==================================================================

async func carre(n: int) -> int {
    sleep(1);
    return n * n;
}

let taches = [];
for i in range(1, 6) {
    taches.add(carre(i));
}

let total = 0;
for t in taches {
    total += await t;
}
println(total);
