// ==================================================================
// Exemple 313 — Erreur dans une tâche, récupérée par join
// Catégorie : Exceptions try/catch/finally
// ------------------------------------------------------------------
// L'exception lancée dans une tâche est relancée chez celui qui attend.
// ------------------------------------------------------------------
// Sortie attendue :
//   récupéré : échec dans la tâche
// ==================================================================

func travail() {
    throw "échec dans la tâche";
}

let t = spawn(travail);
try {
    t.join();
} catch (e) {
    println("récupéré : " + e);
}
