// ==================================================================
// Exemple 541 — Années bissextiles
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : divisible par 4, sauf les siècles non divisibles par 400.
// ------------------------------------------------------------------
// Sortie attendue :
//   1900 : false
//   2000 : true
//   2023 : false
//   2024 : true
// ==================================================================

func bissextile(an: int) -> bool {
    return (an % 4 == 0 && an % 100 != 0) || an % 400 == 0;
}

for an in [1900, 2000, 2023, 2024] {
    println("{} : {}", an, bissextile(an));
}
