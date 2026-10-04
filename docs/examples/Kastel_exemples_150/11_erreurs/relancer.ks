// ====================================================================
// Kastel — Relancer une erreur
// Notions : catch puis throw à nouveau vers l'appelant
// Résultat attendu :
//   niveau 2 : journalisé
//   niveau 1 : disque plein
// ====================================================================

func ecrire() {
    try {
        throw "disque plein";
    } catch (e) {
        println("niveau 2 : journalisé");
        throw e;
    }
}

try {
    ecrire();
} catch (e) {
    println("niveau 1 : {}", e);
}
