// ====================================================================
// Kastel — Supprimer les doublons
// Notions : Set pour dédupliquer, to_list, sort
// Résultat attendu :
//   [1, 2, 3]
// ====================================================================

let doublons = [3, 1, 3, 2, 1, 3];
let vus = Set();

for x in doublons {
    vus.add(x);
}

let uniques = vus.to_list();
uniques.sort();
println(uniques);
