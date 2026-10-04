// ====================================================================
// Kastel — Division par zéro
// Notions : try / catch (e: Err), e.kind
// Résultat attendu :
//   5.0
//   Erreur : DivisionByZero
// ====================================================================

func diviser(a, b) {
    return a / b;
}

try {
    println(diviser(10, 2));
    println(diviser(1, 0));
} catch (e: Err) {
    println("Erreur : {}", e.kind);
}
