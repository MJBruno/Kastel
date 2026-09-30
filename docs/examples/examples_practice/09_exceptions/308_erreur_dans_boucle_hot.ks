// ==================================================================
// Exemple 308 — Erreur dans une boucle rapide
// Catégorie : Exceptions try/catch/finally
// ------------------------------------------------------------------
// Même dans une boucle serrée, un débordement reste attrapable.
// ------------------------------------------------------------------
// Sortie attendue :
//   -1
// ==================================================================

func gros() -> int {
    let i = 9223372036854775800;
    try {
        while true {
            i = i + 1;
        }
    } catch (e: Err) {
        return -1;
    }
    return i;
}

println(gros());
