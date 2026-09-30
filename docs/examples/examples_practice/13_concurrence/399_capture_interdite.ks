// ==================================================================
// Exemple 399 — Pas de capture de variable locale
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// Une tâche n'accède qu'aux variables globales ; les données passent par les arguments.
// ------------------------------------------------------------------
// Sortie attendue :
//   105
// ==================================================================

func travail(base: int, n: int) -> int {
    return base + n;
}

let base = 100;
let t = spawn(travail, base, 5);   // passage explicite par argument
println(t.join());
