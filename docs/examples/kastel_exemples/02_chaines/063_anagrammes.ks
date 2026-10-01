// ==================================================================
// Exemple 063 — Tester deux anagrammes
// Catégorie : Chaînes de caractères
// ------------------------------------------------------------------
// Trier les lettres de chaque mot puis comparer.
// ------------------------------------------------------------------
// Sortie attendue :
//   true
//   false
// ==================================================================

func lettres_triees(s: str) -> str {
    let chars = [];
    for c in s {
        chars.add(c);
    }
    chars.sort();
    return "".join(chars);
}

println(lettres_triees("chien") == lettres_triees("niche"));
println(lettres_triees("chat") == lettres_triees("chien"));
