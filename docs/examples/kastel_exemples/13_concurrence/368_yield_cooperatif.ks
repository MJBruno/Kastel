// ==================================================================
// Exemple 368 — yield : céder la main
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// yield() laisse les autres tâches avancer.
// ------------------------------------------------------------------
// Sortie attendue :
//   6
// ==================================================================

let compteur = 0;

func travailleur() {
    for i in range(3) {
        compteur += 1;
        yield();
    }
}

let a = spawn(travailleur);
let b = spawn(travailleur);
a.join();
b.join();
println(compteur);
