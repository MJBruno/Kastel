// ====================================================================
// Kastel — Liste de tuples
// Notions : parcours d'une liste de paires
// Résultat attendu :
//   1 = un
//   2 = deux
//   3 = trois
// ====================================================================

let paires = [(1, "un"), (2, "deux"), (3, "trois")];

for paire in paires {
    println("{} = {}", paire.get(0), paire.get(1));
}
