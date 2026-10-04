// ====================================================================
// Kastel — Attendre un groupe de tâches
// Notions : wait_group, add, done, wait
// Résultat attendu :
//   3 tâches terminées
// ====================================================================

let groupe = wait_group();
let resultats = [];

func travail(g, sortie, id) {
    sortie.add(id * id);
    g.done();
}

groupe.add(3);
spawn(travail, groupe, resultats, 1);
spawn(travail, groupe, resultats, 2);
spawn(travail, groupe, resultats, 3);

groupe.wait();
println("{} tâches terminées", resultats.size());
