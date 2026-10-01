// ==================================================================
// Exemple 620 — Sous-tableau de somme maximale
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : algorithme de Kadane en une seule passe.
// ------------------------------------------------------------------
// Sortie attendue :
//   6
// ==================================================================

func meilleure_somme(v: List<int>) -> int {
    let courante = v[0];
    let meilleure = v[0];
    for i in range(1, v.size()) {
        courante = max(v[i], courante + v[i]);
        meilleure = max(meilleure, courante);
    }
    return meilleure;
}

println(meilleure_somme([-2, 1, -3, 4, -1, 2, 1, -5, 4]));
