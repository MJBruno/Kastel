// ==================================================================
// Exemple 366 — Plusieurs tâches
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// Lancer plusieurs tâches puis récupérer leurs résultats.
// ------------------------------------------------------------------
// Sortie attendue :
//   [3, 6, 9, 12]
// ==================================================================

func triple(x: int) -> int {
    return x * 3;
}

let taches = [];
for i in range(1, 5) {
    taches.add(spawn(triple, i));
}

let resultats = [];
for t in taches {
    resultats.add(t.join());
}
println(resultats);
