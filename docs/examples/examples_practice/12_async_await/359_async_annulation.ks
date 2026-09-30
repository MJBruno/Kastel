// ==================================================================
// Exemple 359 — Annuler une tâche attendue
// Catégorie : async / await
// ------------------------------------------------------------------
// Celui qui attend reçoit l'erreur TaskCancelled.
// ------------------------------------------------------------------
// Sortie attendue :
//   TaskCancelled
//   cancelled
// ==================================================================

async func lente() -> int {
    sleep(1000);
    return 1;
}

async func attendeur(cible: Task<int>) -> str {
    try {
        await cible;
        return "inattendu";
    } catch (e: Err) {
        return e.kind;
    }
}

let cible = lente();
let a = attendeur(cible);
while a.status() != "waiting" {
    sleep(1);
}
cible.cancel();
println(await a);
println(cible.status());
