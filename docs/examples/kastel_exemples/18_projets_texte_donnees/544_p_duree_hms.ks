// ==================================================================
// Exemple 544 — Secondes vers heures:minutes:secondes
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : division entière et reste.
// ------------------------------------------------------------------
// Sortie attendue :
//   01:02:05
//   00:00:59
//   23:59:59
// ==================================================================

func hms(total: int) -> str {
    let h = idiv(total, 3600);
    let m = idiv(total % 3600, 60);
    let s = total % 60;
    return format("{:02d}:{:02d}:{:02d}", h, m, s);
}

println(hms(3725));
println(hms(59));
println(hms(86399));
