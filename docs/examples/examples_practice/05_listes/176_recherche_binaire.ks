// ==================================================================
// Exemple 176 — Recherche dichotomique
// Catégorie : Listes
// ------------------------------------------------------------------
// Sur une liste triée, diviser l'intervalle par deux à chaque étape.
// ------------------------------------------------------------------
// Sortie attendue :
//   3
//   -1
// ==================================================================

func chercher(v: List<int>, cible: int) -> int {
    let bas = 0;
    let haut = v.size() - 1;
    while bas <= haut {
        let milieu = idiv(bas + haut, 2);
        if v[milieu] == cible {
            return milieu;
        } else if v[milieu] < cible {
            bas = milieu + 1;
        } else {
            haut = milieu - 1;
        }
    }
    return -1;
}

let tri = [1, 3, 5, 7, 9, 11];
println(chercher(tri, 7));
println(chercher(tri, 4));
