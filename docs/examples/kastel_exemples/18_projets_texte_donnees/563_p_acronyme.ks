// ==================================================================
// Exemple 563 — Générer un acronyme
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : la première lettre de chaque mot, en majuscule.
// ------------------------------------------------------------------
// Sortie attendue :
//   PNG
//   LDP
// ==================================================================

func acronyme(texte: str) -> str {
    let res = "";
    for mot in texte.split(" ") {
        if !mot.is_empty() {
            res += mot.char_at(0).upper();
        }
    }
    return res;
}

println(acronyme("Portable Network Graphics"));
println(acronyme("langage de programmation"));
