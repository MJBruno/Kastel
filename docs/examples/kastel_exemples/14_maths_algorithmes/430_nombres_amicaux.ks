// ==================================================================
// Exemple 430 — Somme des diviseurs
// Catégorie : Maths et algorithmes
// ------------------------------------------------------------------
// Fonction utilitaire réutilisable.
// ------------------------------------------------------------------
// Sortie attendue :
//   284
//   220
// ==================================================================

func somme_diviseurs(n: int) -> int {
    let s = 0;
    for d in range(1, n) {
        if n % d == 0 {
            s += d;
        }
    }
    return s;
}

println(somme_diviseurs(220));
println(somme_diviseurs(284));
