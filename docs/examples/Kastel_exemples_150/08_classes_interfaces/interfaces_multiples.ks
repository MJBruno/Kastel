// ====================================================================
// Kastel — Plusieurs interfaces
// Notions : class X: A, B
// Résultat attendu :
//   Robot R2 marche
//   Robot R2 dit bip
// ====================================================================

interface Marcheur {
    func marcher() -> str;
}

interface Parleur {
    func parler() -> str;
}

class Robot: Marcheur, Parleur {
    let nom: str;

    func initialize(nom: str) { self.nom = nom; }
    func marcher() -> str { return "Robot " + self.nom + " marche"; }
    func parler() -> str { return "Robot " + self.nom + " dit bip"; }
}

let r = new Robot("R2");
println(r.marcher());
println(r.parler());
