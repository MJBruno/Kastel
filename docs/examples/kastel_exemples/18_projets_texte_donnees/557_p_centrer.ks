// ==================================================================
// Exemple 557 — Centrer du texte
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : répartir les espaces de chaque côté.
// ------------------------------------------------------------------
// Sortie attendue :
//   [  ok  ]
//   [  abc   ]
// ==================================================================

func centrer(s: str, largeur: int) -> str {
    let reste = largeur - s.size();
    let gauche = idiv(reste, 2);
    return " ".repeat(gauche) + s + " ".repeat(reste - gauche);
}

println("[" + centrer("ok", 6) + "]");
println("[" + centrer("abc", 8) + "]");
