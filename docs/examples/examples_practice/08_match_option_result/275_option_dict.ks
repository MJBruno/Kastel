// ==================================================================
// Exemple 275 — Chercher dans un dict
// Catégorie : match, Option, Result
// ------------------------------------------------------------------
// Emballer un accès risqué dans un Option.
// ------------------------------------------------------------------
// Sortie attendue :
//   2
//   -1
// ==================================================================

let prix = {"pain": 1, "lait": 2};

func prix_de(nom: str) -> Option<int> {
    if prix.contains(nom) {
        return Some(prix[nom]);
    }
    return None;
}

println(prix_de("lait").unwrap_or(-1));
println(prix_de("caviar").unwrap_or(-1));
