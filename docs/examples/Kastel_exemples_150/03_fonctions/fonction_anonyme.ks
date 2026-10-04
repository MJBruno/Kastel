// ====================================================================
// Kastel — Fonction anonyme
// Notions : func(...) { ... } comme valeur
// Résultat attendu :
//   49
//   Bonjour Zoé
// ====================================================================

let carre = func(x) {
    return x * x;
};

let saluer = func(nom) {
    return "Bonjour " + nom;
};

println(carre(7));
println(saluer("Zoé"));
