// ==================================================================
// Exemple 657 — Fan-in : plusieurs producteurs, un consommateur
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : trois producteurs écrivent dans le même canal.
// ------------------------------------------------------------------
// Sortie attendue :
//   18
// ==================================================================

let sortie = channel<int>();

func producteur() {
    for v in [1, 2, 3] {
        sortie.send(v);
    }
}

for i in range(3) {
    spawn(producteur);
}

let total = 0;
for i in range(9) {
    total += sortie.recv();
}
println(total);
