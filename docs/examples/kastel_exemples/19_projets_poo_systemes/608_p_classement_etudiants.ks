// ==================================================================
// Exemple 608 — Classement d'étudiants
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : moyenne par élève puis tri décroissant.
// ------------------------------------------------------------------
// Sortie attendue :
//   1. Ada (17.0)
//   2. Cléo (14.5)
//   3. Bob (11.0)
// ==================================================================

class Etudiant {
    let nom: str;
    let notes: List<int>;

    func initialize(nom: str, notes: List<int>) {
        self.nom = nom;
        self.notes = notes;
    }

    func moyenne() -> float {
        let s = 0;
        for n in self.notes { s += n; }
        return s / self.notes.size();
    }
}

let classe = [
    new Etudiant("Ada", [16, 18]),
    new Etudiant("Bob", [10, 12]),
    new Etudiant("Cléo", [14, 15])
];

// Tri à bulles sur la moyenne, décroissant.
let n = classe.size();
for i in range(n) {
    for j in range(n - 1 - i) {
        if classe[j].moyenne() < classe[j + 1].moyenne() {
            let t = classe[j];
            classe[j] = classe[j + 1];
            classe[j + 1] = t;
        }
    }
}

for i in range(n) {
    println("{}. {} ({:.1f})", i + 1, classe[i].nom, classe[i].moyenne());
}
