// ==================================================================
// Exemple 629 — Maximum sur fenêtre glissante
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : le plus grand élément de chaque fenêtre de taille 3.
// ------------------------------------------------------------------
// Sortie attendue :
//   [3, 3, 5, 5, 6, 7]
// ==================================================================

let v = [1, 3, -1, -3, 5, 3, 6, 7];
let k = 3;
let res = [];
for i in range(v.size() - k + 1) {
    let m = v[i];
    for j in range(1, k) {
        m = max(m, v[i + j]);
    }
    res.add(m);
}
println(res);
