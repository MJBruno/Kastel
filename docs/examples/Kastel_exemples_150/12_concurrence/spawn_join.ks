// ====================================================================
// Kastel — Lancer une tâche
// Notions : spawn(fonction, args...), join()
// Résultat attendu :
//   55
// ====================================================================

func somme_jusqua(n: int) -> int {
    let total = 0;
    for i in range(1, n + 1) {
        total += i;
    }
    return total;
}

let tache = spawn(somme_jusqua, 10);
println(tache.join());
