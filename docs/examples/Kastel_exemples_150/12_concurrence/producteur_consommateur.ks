// ====================================================================
// Kastel — Producteur / consommateur
// Notions : une tâche envoie, le programme principal reçoit
// Résultat attendu :
//   reçu 1
//   reçu 2
//   reçu 3
// ====================================================================

func producteur(c) {
    for i in range(1, 4) {
        c.send(i);
        yield();
    }
}

let canal = channel();
let t = spawn(producteur, canal);

for i in range(3) {
    println("reçu {}", canal.recv());
}

t.join();
