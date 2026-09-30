// ==================================================================
// Exemple 378 — Pipeline à deux étages
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// Une tâche transforme ce qu'une autre produit.
// ------------------------------------------------------------------
// Sortie attendue :
//   2
//   4
//   6
// ==================================================================

let entree = channel<int>();
let sortie = channel<int>();

func source() {
    for i in range(1, 4) {
        entree.send(i);
    }
}

func doubler() {
    for i in range(3) {
        sortie.send(entree.recv() * 2);
    }
}

spawn(source);
spawn(doubler);
println(sortie.recv());
println(sortie.recv());
println(sortie.recv());
