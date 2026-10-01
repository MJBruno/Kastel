// ==================================================================
// Exemple 261 — match sur un booléen
// Catégorie : match, Option, Result
// ------------------------------------------------------------------
// Il faut couvrir true et false (ou utiliser _).
// ------------------------------------------------------------------
// Sortie attendue :
//   oui
//   non
// ==================================================================

func oui_non(b: bool) -> str {
    match b {
        true => { return "oui"; }
        false => { return "non"; }
    }
}

println(oui_non(true));
println(oui_non(false));
