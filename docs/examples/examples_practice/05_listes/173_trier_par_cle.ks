// ==================================================================
// Exemple 173 — Trier des enregistrements
// Catégorie : Listes
// ------------------------------------------------------------------
// Ici avec un tri à bulles écrit à la main, sur une clé.
// ------------------------------------------------------------------
// Sortie attendue :
//   Ada (20)
//   Bob (25)
//   Zoe (30)
// ==================================================================

let gens = [
    { nom: "Zoe", age: 30 },
    { nom: "Ada", age: 20 },
    { nom: "Bob", age: 25 }
];

let n = gens.size();
for i in range(n) {
    for j in range(n - 1 - i) {
        if gens[j].age > gens[j + 1].age {
            let tmp = gens[j];
            gens[j] = gens[j + 1];
            gens[j + 1] = tmp;
        }
    }
}

for g in gens {
    println("{} ({})", g.nom, g.age);
}
