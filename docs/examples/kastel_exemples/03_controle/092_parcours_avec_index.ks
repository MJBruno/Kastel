// ==================================================================
// Exemple 092 — Parcourir avec un index
// Catégorie : Structures de contrôle
// ------------------------------------------------------------------
// Utiliser range(liste.size()) quand on a besoin de la position.
// ------------------------------------------------------------------
// Sortie attendue :
//   0: Ada
//   1: Alan
//   2: Grace
// ==================================================================

let noms = ["Ada", "Alan", "Grace"];
for i in range(noms.size()) {
    println("{}: {}", i, noms[i]);
}
