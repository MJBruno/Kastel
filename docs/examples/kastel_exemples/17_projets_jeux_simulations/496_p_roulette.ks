// ==================================================================
// Exemple 496 — Roulette : couleur d'un numéro
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : les rouges sont un ensemble fixe ; 0 est vert, le reste noir.
// ------------------------------------------------------------------
// Sortie attendue :
//   vert
//   rouge
//   noir
//   rouge
// ==================================================================

let rouges = Set(1, 3, 5, 7, 9, 12, 14, 16, 18, 19, 21, 23, 25, 27, 30, 32, 34, 36);

func couleur(n: int) -> str {
    if n == 0 { return "vert"; }
    return rouges.contains(n) ? "rouge" : "noir";
}

println(couleur(0));
println(couleur(7));
println(couleur(8));
println(couleur(36));
