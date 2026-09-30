// ==================================================================
// Exemple 079 — range avec un pas
// Catégorie : Structures de contrôle
// ------------------------------------------------------------------
// range(debut, fin, pas) permet de sauter des valeurs ou de descendre.
// ------------------------------------------------------------------
// Sortie attendue :
//   0
//   3
//   6
//   9
//   5
//   3
//   1
// ==================================================================

for i in range(0, 10, 3) {
    println(i);          // 0 3 6 9
}
for i in range(5, 0, -2) {
    println(i);          // 5 3 1
}
