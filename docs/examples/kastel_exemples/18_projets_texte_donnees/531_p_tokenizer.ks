// ==================================================================
// Exemple 531 — Découper une expression en jetons
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : nombres, opérateurs et parenthèses.
// ------------------------------------------------------------------
// Sortie attendue :
//   ["12", "+", "34", "*", "(", "5", "-", "6", ")"]
// ==================================================================

func tokeniser(s: str) -> List<str> {
    let res = [];
    let i = 0;
    while i < s.size() {
        let c = s.char_at(i);
        if c.is_digit() {
            let j = i;
            while j < s.size() && s.char_at(j).is_digit() {
                j += 1;
            }
            res.add(s.slice(i, j));
            i = j;
        } else {
            if c != " " {
                res.add(c);
            }
            i += 1;
        }
    }
    return res;
}

println(tokeniser("12+34*(5-6)"));
