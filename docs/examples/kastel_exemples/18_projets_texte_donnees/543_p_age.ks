// ==================================================================
// Exemple 543 — Calcul d'âge
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : différence d'années, moins un si l'anniversaire n'est pas encore passé.
// ------------------------------------------------------------------
// Sortie attendue :
//   36
//   35
// ==================================================================

func age(an: int, mois: int, jour: int, ref_an: int, ref_mois: int, ref_jour: int) -> int {
    let a = ref_an - an;
    if ref_mois < mois || (ref_mois == mois && ref_jour < jour) {
        a -= 1;
    }
    return a;
}

println(age(1990, 5, 15, 2026, 9, 30));
println(age(1990, 12, 25, 2026, 9, 30));
