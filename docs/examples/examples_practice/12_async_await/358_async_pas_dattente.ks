// ==================================================================
// Exemple 358 — Une tâche non attendue s'exécute quand même
// Catégorie : async / await
// ------------------------------------------------------------------
// Sans await, on peut vérifier son état plus tard avec status() ou is_done().
// ------------------------------------------------------------------
// Sortie attendue :
//   done
// ==================================================================

async func silencieuse() -> int {
    return 1;
}

let t = silencieuse();
while !t.is_done() {
    sleep(1);
}
println(t.status());
