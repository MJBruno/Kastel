// ====================================================================
// Kastel — Surcharge de fonctions
// Notions : même nom, nombre de paramètres différent
// Résultat attendu :
//   12
//   12
// ====================================================================

func aire(rayon) {
    return 3 * rayon * rayon;
}

func aire(largeur, hauteur) {
    return largeur * hauteur;
}

println(aire(2));
println(aire(3, 4));
