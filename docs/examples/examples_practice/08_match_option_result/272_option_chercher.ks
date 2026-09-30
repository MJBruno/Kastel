// ==================================================================
// Exemple 272 — Fonction qui peut échouer : Option
// Catégorie : match, Option, Result
// ------------------------------------------------------------------
// Renvoyer Some(...) ou None au lieu d'un code spécial comme -1.
// ------------------------------------------------------------------
// Sortie attendue :
//   Some(1)
//   None
// ==================================================================

func trouver(v: List<int>, cible: int) -> Option<int> {
    for i in range(v.size()) {
        if v[i] == cible {
            return Some(i);
        }
    }
    return None;
}

println(trouver([5, 6, 7], 6));
println(trouver([5, 6, 7], 9));
