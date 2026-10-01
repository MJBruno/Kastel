// ==================================================================
// Exemple 647 — Producteur / consommateur à tampon borné
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : le canal de capacité 2 fait patienter le producteur s'il va trop vite.
// ------------------------------------------------------------------
// Sortie attendue :
//   15
// ==================================================================

let tampon = channel<int>(2);

func producteur() {
    for i in range(1, 6) {
        tampon.send(i);
    }
}

func consommateur() -> int {
    let total = 0;
    for i in range(5) {
        total += tampon.recv();
    }
    return total;
}

let p = spawn(producteur);
let c = spawn(consommateur);
p.join();
println(c.join());
