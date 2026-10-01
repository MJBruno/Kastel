// ==================================================================
// Exemple 660 — Cache protégé par RwLock
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : un écrivain met à jour la valeur, puis des lecteurs la lisent en parallèle.
// ------------------------------------------------------------------
// Sortie attendue :
//   126
// ==================================================================

let verrou = rwlock();
let valeur = 0;

func ecrivain() {
    verrou.write_lock();
    valeur = 42;
    verrou.write_unlock();
}

func lecteur() -> int {
    verrou.read_lock();
    let v = valeur;
    verrou.read_unlock();
    return v;
}

spawn(ecrivain).join();
let lecteurs = [spawn(lecteur), spawn(lecteur), spawn(lecteur)];
let somme = 0;
for l in lecteurs {
    somme += l.join();
}
println(somme);
