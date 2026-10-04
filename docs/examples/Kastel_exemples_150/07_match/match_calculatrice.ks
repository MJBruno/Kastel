// ====================================================================
// Kastel — Mini-calculatrice avec match
// Notions : match sur une chaîne d'opérateur, Result
// Résultat attendu :
//   Ok(7)
//   Ok(12)
//   Err(division par zéro)
//   Err(opérateur inconnu : ^)
// ====================================================================

func calculer(a: int, op: str, b: int) -> Result<int, str> {
    match op {
        "+" => { return Ok(a + b); },
        "*" => { return Ok(a * b); },
        "/" => {
            if b == 0 {
                return Err("division par zéro");
            }
            return Ok(idiv(a, b));
        },
        _ => { return Err("opérateur inconnu : " + op); }
    }
}

println(calculer(3, "+", 4));
println(calculer(3, "*", 4));
println(calculer(3, "/", 0));
println(calculer(3, "^", 4));
