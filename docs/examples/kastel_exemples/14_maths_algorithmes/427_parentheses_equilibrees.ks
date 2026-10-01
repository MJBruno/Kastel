// ==================================================================
// Exemple 427 — Parenthèses équilibrées
// Catégorie : Maths et algorithmes
// ------------------------------------------------------------------
// Une pile qui mémorise les ouvertures.
// ------------------------------------------------------------------
// Sortie attendue :
//   true
//   false
//   false
// ==================================================================

func equilibre(s: str) -> bool {
    let pile = [];
    for c in s {
        if c == "(" {
            pile.add(c);
        } else if c == ")" {
            if pile.is_empty() {
                return false;
            }
            pile.pop();
        }
    }
    return pile.is_empty();
}

println(equilibre("(a + (b * c))"));
println(equilibre("((a + b)"));
println(equilibre(")("));
