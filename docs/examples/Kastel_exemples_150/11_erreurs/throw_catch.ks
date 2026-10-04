// ====================================================================
// Kastel — throw et catch
// Notions : lever n'importe quelle valeur, la rattraper avec catch (e)
// Résultat attendu :
//   Résultat : 5
//   Erreur : age invalide
// ====================================================================

func verifier(age) {
    if age < 0 {
        throw "age invalide";
    }
    return age;
}

try {
    println("Résultat : {}", verifier(5));
    println("Résultat : {}", verifier(-1));
} catch (e) {
    println("Erreur : {}", e);
}
