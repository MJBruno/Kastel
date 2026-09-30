// ==================================================================
// Exemple 371 — Ordre garanti
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// Les messages arrivent dans l'ordre d'envoi.
// ------------------------------------------------------------------
// Sortie attendue :
//   [0, 1, 2, 3, 4]
// ==================================================================

let c = channel<int>();

func envoyer() {
    for i in range(5) {
        c.send(i);
    }
}

let t = spawn(envoyer);
let recus = [];
for i in range(5) {
    recus.add(c.recv());
}
println(recus);
