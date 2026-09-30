// ==================================================================
// Exemple 352 — Chaîne d'await
// Catégorie : async / await
// ------------------------------------------------------------------
// Chaque tâche attend la suivante.
// ------------------------------------------------------------------
// Sortie attendue :
//   50
// ==================================================================

async func chaine(n: int) -> int {
    if n == 0 {
        return 0;
    }
    let suivante = chaine(n - 1);
    return (await suivante) + 1;
}

println(await chaine(50));
