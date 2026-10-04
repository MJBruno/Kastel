// ====================================================================
// Kastel — Parcourir une chaîne
// Notions : for sur les caractères
// Résultat attendu :
//   K-A-S-T-E-L
// ====================================================================

let lettres = [];

for c in "kastel" {
    lettres.add(c.upper());
}

println(lettres.join("-"));
