// ====================================================================
// Kastel — Result avec match
// Notions : retourner Ok ou Err, puis traiter les deux cas
// Résultat attendu :
//   résultat = 2.5
//   erreur : division par zéro
// ====================================================================

func diviser(a: float, b: float) -> Result<float, str> {
    if b == 0.0 {
        return Err("division par zéro");
    }
    return Ok(a / b);
}

match diviser(10.0, 4.0) {
    Ok(v) => { println("résultat = {}", v); },
    Err(e) => { println("erreur : {}", e); }
}

match diviser(1.0, 0.0) {
    Ok(v) => { println("résultat = {}", v); },
    Err(e) => { println("erreur : {}", e); }
}
