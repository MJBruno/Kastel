// ==================================================================
// Exemple 549 — Indice de masse corporelle
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : IMC = poids / taille², avec catégorie.
// ------------------------------------------------------------------
// Sortie attendue :
//   22.9 : normal
// ==================================================================

func imc(poids: float, taille: float) -> float {
    return poids / (taille * taille);
}

func categorie(v: float) -> str {
    if v < 18.5 { return "maigreur"; }
    if v < 25.0 { return "normal"; }
    if v < 30.0 { return "surpoids"; }
    return "obésité";
}

let v = imc(70.0, 1.75);
println("{:.1f} : {}", v, categorie(v));
