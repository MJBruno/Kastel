// ==================================================================
// Exemple 351 — Lancer plusieurs tâches puis attendre
// Catégorie : async / await
// ------------------------------------------------------------------
// Les tâches démarrent dès l'appel : les attentes se chevauchent.
// ------------------------------------------------------------------
// Sortie attendue :
//   100
//   200
//   300
// ==================================================================

async func travail(id: int, duree: int) -> int {
    sleep(duree);
    return id * 100;
}

let a = travail(1, 5);
let b = travail(2, 5);
let c = travail(3, 5);

println(await a);
println(await b);
println(await c);
