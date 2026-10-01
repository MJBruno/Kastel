// ==================================================================
// Exemple 560 — Plus long palindrome d'un texte
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : étendre autour de chaque centre possible.
// ------------------------------------------------------------------
// Sortie attendue :
//   bab
//   bb
// ==================================================================

func etendre(s: str, g: int, d: int) -> str {
    while g >= 0 && d < s.size() && s.char_at(g) == s.char_at(d) {
        g -= 1;
        d += 1;
    }
    return s.slice(g + 1, d);
}

func plus_long(s: str) -> str {
    let meilleur = "";
    for i in range(s.size()) {
        let impair = etendre(s, i, i);
        let pair = etendre(s, i, i + 1);
        if impair.size() > meilleur.size() { meilleur = impair; }
        if pair.size() > meilleur.size() { meilleur = pair; }
    }
    return meilleur;
}

println(plus_long("babad"));
println(plus_long("cbbd"));
