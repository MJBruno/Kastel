// ====================================================================
// Kastel — Dictionnaires imbriqués
// Notions : d[a][b], affectation imbriquée
// Résultat attendu :
//   25
//   23
// ====================================================================

let ecole = {
    "classe_a": {"eleves": 25, "salle": 12},
    "classe_b": {"eleves": 22, "salle": 14}
};

println(ecole["classe_a"]["eleves"]);

ecole["classe_b"]["eleves"] = 23;
println(ecole["classe_b"]["eleves"]);
