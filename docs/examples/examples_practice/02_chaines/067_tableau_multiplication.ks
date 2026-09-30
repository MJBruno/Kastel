// ==================================================================
// Exemple 067 — Table de multiplication formatée
// Catégorie : Chaînes de caractères
// ------------------------------------------------------------------
// Le format {:>3} aligne les nombres à droite sur 3 colonnes.
// ------------------------------------------------------------------
// Sortie attendue :
//     1  2  3  4  5
//     2  4  6  8 10
//     3  6  9 12 15
// ==================================================================

for i in range(1, 4) {
    let ligne = "";
    for j in range(1, 6) {
        ligne += format("{:>3}", i * j);
    }
    println(ligne);
}
