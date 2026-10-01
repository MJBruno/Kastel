// ==================================================================
// Exemple 542 — Jour de la semaine (Zeller)
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : retrouver le jour d'une date du calendrier grégorien.
// ------------------------------------------------------------------
// Sortie attendue :
//   mercredi
//   samedi
// ==================================================================

let noms = ["samedi", "dimanche", "lundi", "mardi", "mercredi", "jeudi", "vendredi"];

func jour_semaine(an: int, mois: int, jour: int) -> str {
    let m = mois;
    let y = an;
    if m < 3 {
        m += 12;
        y -= 1;
    }
    let k = y % 100;
    let j = idiv(y, 100);
    let h = (jour + idiv(13 * (m + 1), 5) + k + idiv(k, 4) + idiv(j, 4) + 5 * j) % 7;
    return noms[h];
}

println(jour_semaine(2026, 9, 30));
println(jour_semaine(2000, 1, 1));
