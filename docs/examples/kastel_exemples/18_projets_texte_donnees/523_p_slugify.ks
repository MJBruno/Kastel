// ==================================================================
// Exemple 523 — Slugify : texte vers URL
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : minuscules, tirets à la place des séparateurs, sans tiret en bord.
// ------------------------------------------------------------------
// Sortie attendue :
//   bonjour-le-monde
//   hello-world
// ==================================================================

func slug(texte: str) -> str {
    let res = "";
    for c in texte.lower() {
        if c.is_alphanumeric() {
            res += c;
        } else if res != "" && !res.ends_with("-") {
            res += "-";
        }
    }
    if res.ends_with("-") {
        res = res.slice(0, res.size() - 1);
    }
    return res;
}

println(slug("Bonjour le Monde !"));
println(slug("  Hello   World  "));
