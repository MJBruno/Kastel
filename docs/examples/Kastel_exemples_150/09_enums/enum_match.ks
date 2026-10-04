// ====================================================================
// Kastel — Énumération et match
// Notions : un pattern par variant
// Résultat attendu :
//   Feu rouge : stop
//   Feu vert : passez
//   Feu orange : ralentissez
// ====================================================================

enum Feu {
    Rouge,
    Orange,
    Vert
}

func action(f: Feu) {
    match f {
        Feu.Rouge => { println("Feu rouge : stop"); },
        Feu.Orange => { println("Feu orange : ralentissez"); },
        Feu.Vert => { println("Feu vert : passez"); }
    }
}

action(Feu.Rouge);
action(Feu.Vert);
action(Feu.Orange);
