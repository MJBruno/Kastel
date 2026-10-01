// ==================================================================
// Exemple 369 — sleep dans une tâche
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// sleep(ms) suspend seulement la tâche courante.
// ------------------------------------------------------------------
// Sortie attendue :
//   terminé
// ==================================================================

func lent() -> str {
    sleep(20);
    return "terminé";
}

let t = spawn(lent);
println(t.join());
