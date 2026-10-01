// ==================================================================
// Exemple 621 — Tous les sous-ensembles
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : chaque bit d'un masque décide si l'élément est présent.
// ------------------------------------------------------------------
// Sortie attendue :
//   8
//   [[], [1], [2], [1, 2], [3], [1, 3], [2, 3], [1, 2, 3]]
// ==================================================================

let v = [1, 2, 3];
let tous = [];
for masque in range(1 << v.size()) {
    let sous = [];
    for i in range(v.size()) {
        if ((masque >> i) & 1) == 1 {
            sous.add(v[i]);
        }
    }
    tous.add(sous);
}
println(tous.size());
println(tous);
