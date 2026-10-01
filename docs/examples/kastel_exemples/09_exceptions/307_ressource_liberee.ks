// ==================================================================
// Exemple 307 — Libérer une ressource dans finally
// Catégorie : Exceptions try/catch/finally
// ------------------------------------------------------------------
// Le motif classique : ouvrir, utiliser, toujours fermer.
// ------------------------------------------------------------------
// Sortie attendue :
//   problème pendant l'utilisation
//   0
// ==================================================================

let ouvertes = 0;

func utiliser(echec: bool) {
    ouvertes += 1;                 // « ouvrir »
    try {
        if echec {
            throw "problème pendant l'utilisation";
        }
    } finally {
        ouvertes -= 1;             // « fermer » quoi qu'il arrive
    }
}

utiliser(false);
try {
    utiliser(true);
} catch (e) {
    println(e);
}
println(ouvertes);
