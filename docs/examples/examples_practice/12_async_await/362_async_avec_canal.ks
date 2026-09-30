// ==================================================================
// Exemple 362 — async et canal ensemble
// Catégorie : async / await
// ------------------------------------------------------------------
// Un producteur envoie, un consommateur additionne.
// ------------------------------------------------------------------
// Sortie attendue :
//   15
// ==================================================================

let canal = channel<int>();

async func producteur() {
    for i in range(1, 6) {
        canal.send(i);
    }
}

async func consommateur() -> int {
    let total = 0;
    for i in range(5) {
        total += canal.recv();
    }
    return total;
}

let p = producteur();
let c = consommateur();
await p;
println(await c);
