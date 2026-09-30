// ==================================================================
// Exemple 423 — Recherche linéaire avec Option
// Catégorie : Maths et algorithmes
// ------------------------------------------------------------------
// Parcourir jusqu'à trouver ; None sinon.
// ------------------------------------------------------------------
// Sortie attendue :
//   Some(2)
//   None
// ==================================================================

func chercher(v: List<str>, cible: str) -> Option<int> {
    for i in range(v.size()) {
        if v[i] == cible {
            return Some(i);
        }
    }
    return None;
}

let mots = ["a", "b", "c"];
println(chercher(mots, "c"));
println(chercher(mots, "z"));
