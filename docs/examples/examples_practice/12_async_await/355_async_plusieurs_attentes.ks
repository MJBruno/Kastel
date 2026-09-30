// ==================================================================
// Exemple 355 — Plusieurs tâches attendent la même
// Catégorie : async / await
// ------------------------------------------------------------------
// Une tâche terminée peut être attendue plusieurs fois.
// ------------------------------------------------------------------
// Sortie attendue :
//   8
//   9
// ==================================================================

async func produire() -> int {
    sleep(2);
    return 7;
}

async func consommer(cible: Task<int>, k: int) -> int {
    let v = await cible;
    return v + k;
}

let source = produire();
let a = consommer(source, 1);
let b = consommer(source, 2);
println(await a);
println(await b);
