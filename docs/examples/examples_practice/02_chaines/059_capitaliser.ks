// ==================================================================
// Exemple 059 — Mettre la première lettre en majuscule
// Catégorie : Chaînes de caractères
// ------------------------------------------------------------------
// Combiner char_at, upper et slice.
// ------------------------------------------------------------------
// Sortie attendue :
//   Kastel
//   Bonjour le monde
// ==================================================================

func capitaliser(s: str) -> str {
    if s.is_empty() {
        return s;
    }
    return s.char_at(0).upper() + s.slice(1, s.size());
}

println(capitaliser("kastel"));
println(capitaliser("bonjour le monde"));
