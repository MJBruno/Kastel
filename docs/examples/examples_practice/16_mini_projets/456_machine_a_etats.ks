// ==================================================================
// Exemple 456 — Machine à états avec enum
// Catégorie : Mini-projets
// ------------------------------------------------------------------
// Un feu tricolore qui passe d'un état au suivant.
// ------------------------------------------------------------------
// Sortie attendue :
//   rouge
//   vert
//   orange
//   rouge
// ==================================================================

enum Feu {
    Rouge,
    Vert,
    Orange

    func suivant() -> Feu {
        match self {
            Feu.Rouge => { return Feu.Vert; }
            Feu.Vert => { return Feu.Orange; }
            Feu.Orange => { return Feu.Rouge; }
        }
    }
}

func nom(f: Feu) -> str {
    match f {
        Feu.Rouge => { return "rouge"; }
        Feu.Vert => { return "vert"; }
        Feu.Orange => { return "orange"; }
    }
}

let f = Feu.Rouge;
for i in range(4) {
    println(nom(f));
    f = f.suivant();
}
