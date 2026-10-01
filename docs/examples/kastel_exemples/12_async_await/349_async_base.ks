// ==================================================================
// Exemple 349 — async func et await
// Catégorie : async / await
// ------------------------------------------------------------------
// Une fonction async démarre comme tâche et renvoie un Task<T> ; await donne le résultat.
// ------------------------------------------------------------------
// Sortie attendue :
//   42
//   done
// ==================================================================

async func calculer(x: int) -> int {
    sleep(1);
    return x * 2;
}

let tache: Task<int> = calculer(21);
let resultat = await tache;
println(resultat);
println(tache.status());
