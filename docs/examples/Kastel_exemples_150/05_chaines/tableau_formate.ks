// ====================================================================
// Kastel — Tableau aligné
// Notions : {:<8} et {:>6.2f} dans une boucle
// Résultat attendu :
//   Pomme     1.50
//   Banane    0.25
//   Ananas    3.00
// ====================================================================

let produits = [("Pomme", 1.5), ("Banane", 0.25), ("Ananas", 3.0)];

for p in produits {
    println("{:<8}{:>6.2f}", p.get(0), p.get(1));
}
