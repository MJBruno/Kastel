// ====================================================================
// Kastel — Composition plutôt qu'héritage
// Notions : un objet qui contient d'autres objets
// Résultat attendu :
//   Voiture de 4 roues, moteur 90 ch
// ====================================================================

class Moteur {
    let puissance: int;
    func initialize(puissance: int) { self.puissance = puissance; }
    func chevaux() -> int { return self.puissance; }
}

class Voiture {
    let moteur: Moteur;
    let roues: int;

    func initialize(moteur: Moteur, roues: int) {
        self.moteur = moteur;
        self.roues = roues;
    }

    func description() -> str {
        return "Voiture de " + str(self.roues) + " roues, moteur "
            + str(self.moteur.chevaux()) + " ch";
    }
}

let v = new Voiture(new Moteur(90), 4);
println(v.description());
