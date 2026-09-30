// ==================================================================
// Exemple 364 — finally dans une fonction async
// Catégorie : async / await
// ------------------------------------------------------------------
// Le bloc finally s'exécute à la fin de la tâche.
// ------------------------------------------------------------------
// Sortie attendue :
//   1
//   ["nettoyé"]
// ==================================================================

let journal = [];

async func avec_nettoyage() -> int {
    try {
        return 1;
    } finally {
        journal.add("nettoyé");
    }
}

println(await avec_nettoyage());
println(journal);
