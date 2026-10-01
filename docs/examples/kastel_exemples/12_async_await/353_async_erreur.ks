// ==================================================================
// Exemple 353 — Une erreur remonte par await
// Catégorie : async / await
// ------------------------------------------------------------------
// Une exception dans la tâche est levée au niveau de l'await.
// ------------------------------------------------------------------
// Sortie attendue :
//   attrapé : problème async
// ==================================================================

async func echoue() -> int {
    throw "problème async";
}

try {
    let v = await echoue();
    println(v);
} catch (e) {
    println("attrapé : " + e);
}
