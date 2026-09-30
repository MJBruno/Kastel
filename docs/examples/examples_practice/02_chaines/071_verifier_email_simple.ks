// ==================================================================
// Exemple 071 — Vérification très simple d'un email
// Catégorie : Chaînes de caractères
// ------------------------------------------------------------------
// contains et index_of pour un contrôle minimal (pas une vraie validation).
// ------------------------------------------------------------------
// Sortie attendue :
//   true
//   false
// ==================================================================

func email_plausible(s: str) -> bool {
    let arobase = s.index_of("@");
    return arobase > 0 && s.last_index_of(".") > arobase;
}

println(email_plausible("ada@exemple.fr"));
println(email_plausible("ada.exemple.fr"));
