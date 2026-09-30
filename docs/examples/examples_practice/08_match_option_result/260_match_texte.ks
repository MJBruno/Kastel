// ==================================================================
// Exemple 260 — match sur des textes
// Catégorie : match, Option, Result
// ------------------------------------------------------------------
// Les motifs peuvent être des chaînes.
// ------------------------------------------------------------------
// Sortie attendue :
//   démarrage
//   inconnue
// ==================================================================

func commande(c: str) -> str {
    match c {
        "start" => { return "démarrage"; }
        "stop" => { return "arrêt"; }
        _ => { return "inconnue"; }
    }
}

println(commande("start"));
println(commande("boum"));
