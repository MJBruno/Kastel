// ====================================================================
// Kastel — Exclusion mutuelle
// Notions : mutex(), lock, unlock autour d'une section critique
// Résultat attendu :
//   total = 12
// ====================================================================

let verrou = mutex();
let compteur = [0];

func travailleur(m, partage) {
    for i in range(3) {
        m.lock();
        let valeur = partage[0];
        yield();                      // une autre tâche peut s'intercaler
        partage[0] = valeur + 1;
        m.unlock();
    }
}

let taches = [];
for i in range(4) {
    taches.add(spawn(travailleur, verrou, compteur));
}

for t in taches {
    t.join();
}

println("total = {}", compteur[0]);
