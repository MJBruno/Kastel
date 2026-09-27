// examples/constructors_demo.ks
//
// Constructeurs : la méthode `initialize`.
//   - elle peut être SURCHARGÉE par nombre de paramètres ;
//   - sans `initialize`, un constructeur par défaut implicite (sans
//     paramètre) est utilisé, comme `constructor` en TypeScript ou PHP ;
//   - une classe dérivée hérite des constructeurs de sa classe de base.
// Lancer avec : kastel examples/constructors_demo.ks

// -- Constructeur par défaut implicite --------------------------------------
class Vide {
    func salut() -> str {
        return "salut";
    }
}

let v = new Vide();                     // aucun initialize déclaré
println(v.salut());                     // salut
// new Vide(1);                         // Erreur : le constructeur par défaut
//                                      // n'accepte aucun argument

// Les valeurs initiales des champs sont appliquées avant le constructeur,
// y compris avec le constructeur par défaut.
class Compteur {
    public let total: int = 10;

    func incrementer() -> int {
        this.total = this.total + 1;
        return this.total;
    }
}

let c = new Compteur();
println(c.incrementer());               // 11

// -- Constructeur surchargé ---------------------------------------------------
class Point {
    func initialize() {
        this.x = 0;
        this.y = 0;
    }

    func initialize(x: int) {
        this.x = x;
        this.y = 0;
    }

    func initialize(x: int, y: int) {
        this.x = x;
        this.y = y;
    }

    func to_tuple() -> Tuple<int, int> {
        return (this.x, this.y);
    }
}

println(new Point().to_tuple());        // (0, 0)
println(new Point(5).to_tuple());       // (5, 0)
println(new Point(1, 2).to_tuple());    // (1, 2)
// new Point(1, 2, 3);                  // Erreur : aucun constructeur à 3 arguments

// -- Héritage des constructeurs ------------------------------------------------
class Personne {
    public let niveau: int = 1;

    func initialize(nom: str) {
        this.nom = nom;
    }

    func nom_complet() -> str {
        return this.nom;
    }
}

// Aucun initialize : hérite de celui de Personne.
class Visiteur: Personne {
}

let visiteur = new Visiteur("Alice");
println(visiteur.nom_complet());        // Alice
println(visiteur.niveau);               // 1

// Son propre initialize, qui délègue à celui de la classe de base.
class Employe: Personne {
    func initialize(nom: str, poste: str) {
        base.initialize(nom);
        this.poste = poste;
    }

    func description() -> str {
        return this.nom + " (" + this.poste + ")";
    }
}

let employe = new Employe("Bruno", "ingénieur");
println(employe.description());         // Bruno (ingénieur)
println(employe.niveau);                // 1
