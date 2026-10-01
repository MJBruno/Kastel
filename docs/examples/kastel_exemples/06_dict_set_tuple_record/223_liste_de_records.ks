// ==================================================================
// Exemple 223 — Liste de records
// Catégorie : Dict, Set, Tuple, Record
// ------------------------------------------------------------------
// Calculer sur des données structurées.
// ------------------------------------------------------------------
// Sortie attendue :
//   37
// ==================================================================

let produits = [
    { nom: "stylo", prix: 2 },
    { nom: "cahier", prix: 5 },
    { nom: "sac", prix: 30 }
];
let total = 0;
for p in produits {
    total += p.prix;
}
println(total);
