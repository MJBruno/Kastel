// ====================================================================
// Kastel — Intercepter une erreur d'exécution
// Notions : catch (e: Err), e.kind
// Résultat attendu :
//   DivisionByZero
//   ArrayIndexOutOfBounds
// ====================================================================

try {
    let x = 10 / 0;
    println(x);
} catch (e: Err) {
    println(e.kind);
}

try {
    let liste = [1, 2, 3];
    println(liste[10]);
} catch (e: Err) {
    println(e.kind);
}
