// ==================================================================
// Exemple 281 — Result avec match
// Catégorie : match, Option, Result
// ------------------------------------------------------------------
// Deux bras : Ok(x) et Err(e).
// ------------------------------------------------------------------
// Sortie attendue :
//   âge : 25
//   erreur : saisie invalide
// ==================================================================

func lire_age(s: str) -> Result<int, str> {
    if !s.is_digit() {
        return Err("saisie invalide");
    }
    return Ok(s.to_int());
}

for entree in ["25", "abc"] {
    match lire_age(entree) {
        Ok(age) => { println("âge : {}", age); }
        Err(e) => { println("erreur : {}", e); }
    }
}
