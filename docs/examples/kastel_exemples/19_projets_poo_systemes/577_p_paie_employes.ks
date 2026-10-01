// ==================================================================
// Exemple 577 — Paie des employés
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : un manager hérite d'Employe et redéfinit le calcul de paie.
// ------------------------------------------------------------------
// Sortie attendue :
//   Ada : 2000
//   Bob : 3500
//   masse salariale : 5500
// ==================================================================

class Employe {
    let nom: str = "";
    let salaire: int = 0;

    func paie() -> int {
        return self.salaire;
    }
}

class Manager: Employe {
    func paie() -> int {
        return self.salaire + 500;      // prime de management
    }
}

let a = new Employe();
a.nom = "Ada";
a.salaire = 2000;

let b = new Manager();
b.nom = "Bob";
b.salaire = 3000;

let equipe = [a, b];
let masse = 0;
for e in equipe {
    println("{} : {}", e.nom, e.paie());
    masse += e.paie();
}
println("masse salariale : {}", masse);
