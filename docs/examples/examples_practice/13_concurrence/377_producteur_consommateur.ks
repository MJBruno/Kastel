// ==================================================================
// Exemple 377 — Producteur / consommateur
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// Un producteur envoie, un consommateur additionne.
// ------------------------------------------------------------------
// Sortie attendue :
//   55
// ==================================================================

let c = channel<int>();

func producteur() {
    for i in range(1, 11) {
        c.send(i);
    }
}

func consommateur() -> int {
    let total = 0;
    for i in range(10) {
        total += c.recv();
    }
    return total;
}

let p = spawn(producteur);
let q = spawn(consommateur);
p.join();
println(q.join());
