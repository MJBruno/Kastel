// ====================================================================
// Kastel — Fonctions génériques
// Notions : <T>, List<T>
// Résultat attendu :
//   42
//   texte
//   10
//   a
// ====================================================================

func identite<T>(valeur: T) -> T {
    return valeur;
}

func premier<T>(elements: List<T>) -> T {
    return elements[0];
}

println(identite(42));
println(identite("texte"));
println(premier([10, 20, 30]));
println(premier(["a", "b"]));
