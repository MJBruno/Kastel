// ==================================================================
// Exemple 545 — Afficher le calendrier d'un mois
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : septembre 2026 commence un mardi et compte 30 jours.
// ------------------------------------------------------------------
// Sortie attendue :
//   Lu Ma Me Je Ve Sa Di
//       1  2  3  4  5  6
//    7  8  9 10 11 12 13
//   14 15 16 17 18 19 20
//   21 22 23 24 25 26 27
//   28 29 30
// ==================================================================

let decalage = 1;     // 0 = lundi, 1 = mardi...
let jours = 30;

println("Lu Ma Me Je Ve Sa Di");
let cellules = [];
for i in range(decalage) {
    cellules.add("  ");
}
for j in range(1, jours + 1) {
    cellules.add(format("{:>2}", j));
    if cellules.size() == 7 {
        println(" ".join(cellules));
        cellules = [];
    }
}
if !cellules.is_empty() {
    println(" ".join(cellules));
}
