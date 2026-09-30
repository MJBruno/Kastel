// ==================================================================
// Exemple 453 — Calculatrice en notation polonaise inversée
// Catégorie : Mini-projets
// ------------------------------------------------------------------
// Une pile évalue « 3 4 + 2 * ».
// ------------------------------------------------------------------
// Sortie attendue :
//   Ok(14)
//   Ok(14)
//   Err(pile insuffisante)
//   Err(opérateur inconnu : ^)
// ==================================================================

func calculer(expression: str) -> Result<int, str> {
    let pile = [];
    for jeton in expression.split(" ") {
        if jeton.is_digit() {
            pile.add(jeton.to_int());
        } else {
            if pile.size() < 2 {
                return Err("pile insuffisante");
            }
            let b = pile.pop();
            let a = pile.pop();
            match jeton {
                "+" => { pile.add(a + b); }
                "-" => { pile.add(a - b); }
                "*" => { pile.add(a * b); }
                _ => { return Err("opérateur inconnu : " + jeton); }
            }
        }
    }
    return Ok(pile.pop());
}

println(calculer("3 4 + 2 *"));
println(calculer("5 1 2 + 4 * + 3 -"));
println(calculer("1 +"));
println(calculer("2 3 ^"));
