// ====================================================================
// Kastel — Tâches async imbriquées
// Notions : une tâche qui attend une autre tâche
// Résultat attendu :
//   24
// ====================================================================

async func fondation() -> int {
    return 6;
}

async func quadruple() -> int {
    let t = fondation();
    let v = await t;
    return v * 4;
}

println(await quadruple());
