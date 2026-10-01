// ==================================================================
// Exemple 365 — spawn et join
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// spawn(f, args...) lance une tâche ; join() attend et renvoie son résultat.
// ------------------------------------------------------------------
// Sortie attendue :
//   144
// ==================================================================

func calculer(x: int) -> int {
    return x * x;
}

let t = spawn(calculer, 12);
println(t.join());
