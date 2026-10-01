// ==================================================================
// Exemple 273 — Option avec match
// Catégorie : match, Option, Result
// ------------------------------------------------------------------
// Le motif Some(x) extrait la valeur, None couvre l'absence.
// ------------------------------------------------------------------
// Sortie attendue :
//   valeur : 9
//   aucune valeur
// ==================================================================

func afficher(o: Option<int>) {
    match o {
        Some(x) => { println("valeur : {}", x); }
        None => { println("aucune valeur"); }
    }
}

afficher(Some(9));
afficher(None);
