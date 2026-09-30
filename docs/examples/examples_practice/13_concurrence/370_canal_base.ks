// ==================================================================
// Exemple 370 — Canal : send et recv
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// channel<T>() relie des tâches ; recv() attend une valeur.
// ------------------------------------------------------------------
// Sortie attendue :
//   10
//   20
// ==================================================================

let c = channel<int>();

func emetteur() {
    c.send(10);
    c.send(20);
}

let t = spawn(emetteur);
println(c.recv());
println(c.recv());
t.join();
