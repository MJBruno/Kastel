// ==================================================================
// Exemple 424 — Triangle de Pascal
// Catégorie : Maths et algorithmes
// ------------------------------------------------------------------
// Chaque ligne se déduit de la précédente.
// ------------------------------------------------------------------
// Sortie attendue :
//   [1]
//   [1, 1]
//   [1, 2, 1]
//   [1, 3, 3, 1]
//   [1, 4, 6, 4, 1]
// ==================================================================

let ligne = [1];
for i in range(5) {
    println(ligne);
    let suivante = [1];
    for j in range(ligne.size() - 1) {
        suivante.add(ligne[j] + ligne[j + 1]);
    }
    suivante.add(1);
    ligne = suivante;
}
