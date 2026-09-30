// ==================================================================
// Exemple 360 — await sur autre chose qu'une tâche
// Catégorie : async / await
// ------------------------------------------------------------------
// Le type est vérifié : await 42 est refusé à la compilation.
// ------------------------------------------------------------------
// Sortie attendue :
//   42
// ==================================================================

// let x: int = await 42;    // refusé : 42 n'est pas un Task<T>

async func ok() -> int { return 42; }
let x: int = await ok();
println(x);
