// ==================================================================
// Exemple 248 — Enum et match
// Catégorie : Classes, interfaces, enums, alias
// ------------------------------------------------------------------
// match sur un enum : un bras par valeur.
// ------------------------------------------------------------------
// Sortie attendue :
//   stop
//   passer
// ==================================================================

enum Feu {
    Rouge,
    Orange,
    Vert
}

func action(f: Feu) -> str {
    match f {
        Feu.Rouge => { return "stop"; }
        Feu.Orange => { return "ralentir"; }
        Feu.Vert => { return "passer"; }
    }
}

println(action(Feu.Rouge));
println(action(Feu.Vert));
