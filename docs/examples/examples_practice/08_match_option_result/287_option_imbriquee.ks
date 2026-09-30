// ==================================================================
// Exemple 287 — Option imbriqué avec match
// Catégorie : match, Option, Result
// ------------------------------------------------------------------
// Un motif peut descendre dans plusieurs niveaux.
// ------------------------------------------------------------------
// Sortie attendue :
//   valeur 3
//   vide à l'intérieur
//   rien
// ==================================================================

func profondeur(o: Option<Option<int>>) -> str {
    match o {
        Some(Some(x)) => { return "valeur " + str(x); }
        Some(None) => { return "vide à l'intérieur"; }
        None => { return "rien"; }
    }
}

println(profondeur(Some(Some(3))));
println(profondeur(Some(None)));
println(profondeur(None));
