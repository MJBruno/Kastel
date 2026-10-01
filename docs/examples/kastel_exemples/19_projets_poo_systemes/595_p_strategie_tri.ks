// ==================================================================
// Exemple 595 — Patron Stratégie
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : un même tri paramétré par une fonction de comparaison.
// ------------------------------------------------------------------
// Sortie attendue :
//   [1, 2, 3]
//   [3, 2, 1]
// ==================================================================

func trier(v: List<int>, avant) -> List<int> {
    let res = v.copy();
    for i in range(1, res.size()) {
        let cle = res[i];
        let j = i - 1;
        while j >= 0 && avant(cle, res[j]) {
            res[j + 1] = res[j];
            j -= 1;
        }
        res[j + 1] = cle;
    }
    return res;
}

let donnees = [3, 1, 2];
println(trier(donnees, (a, b) => a < b));
println(trier(donnees, (a, b) => a > b));
