// ==================================================================
// Exemple 354 — Erreur runtime dans une tâche
// Catégorie : async / await
// ------------------------------------------------------------------
// Une division par zéro dans la tâche est signalée à l'attente.
// ------------------------------------------------------------------
// Sortie attendue :
//   DivisionByZero
// ==================================================================

async func diviser(a: int, b: int) -> float {
    return a / b;
}

try {
    await diviser(1, 0);
} catch (e: Err) {
    println(e.kind);
}
