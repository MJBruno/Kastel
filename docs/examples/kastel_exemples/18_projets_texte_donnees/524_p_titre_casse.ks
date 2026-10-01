// ==================================================================
// Exemple 524 — Mettre un titre en majuscules initiales
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : chaque mot commence par une majuscule.
// ------------------------------------------------------------------
// Sortie attendue :
//   Le Petit Prince
//   Le Seigneur Des Anneaux
// ==================================================================

func titre(texte: str) -> str {
    let mots = [];
    for m in texte.split(" ") {
        if m.is_empty() { continue; }
        mots.add(m.char_at(0).upper() + m.slice(1, m.size()).lower());
    }
    return " ".join(mots);
}

println(titre("le petit prince"));
println(titre("LE  SEIGNEUR des anneaux"));
