// ==================================================================
// Exemple 256 — match sur des valeurs
// Catégorie : match, Option, Result
// ------------------------------------------------------------------
// Chaque bras est un motif ; _ attrape tout le reste.
// ------------------------------------------------------------------
// Sortie attendue :
//   mardi
//   autre
// ==================================================================

func nom_du_jour(n: int) -> str {
    match n {
        1 => { return "lundi"; }
        2 => { return "mardi"; }
        3 => { return "mercredi"; }
        _ => { return "autre"; }
    }
}

println(nom_du_jour(2));
println(nom_du_jour(9));
