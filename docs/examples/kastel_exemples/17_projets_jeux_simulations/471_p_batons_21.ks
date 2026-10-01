// ==================================================================
// Exemple 471 — Jeu des 21 bâtons
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : on retire 1 à 3 bâtons, celui qui prend le dernier perd ; l'ordinateur joue pour laisser un multiple de 4 plus 1.
// ------------------------------------------------------------------
// Sortie attendue :
//   humain prend 1 -> reste 20
//   ordi prend 3 -> reste 17
//   humain prend 2 -> reste 15
//   ordi prend 2 -> reste 13
//   humain prend 3 -> reste 10
//   ordi prend 1 -> reste 9
//   humain prend 2 -> reste 7
//   ordi prend 2 -> reste 5
//   humain prend 1 -> reste 4
//   ordi prend 3 -> reste 1
//   l'humain doit prendre le dernier
// ==================================================================

func coup_ordi(reste: int) -> int {
    let c = (reste - 1) % 4;
    return c == 0 ? 1 : c;
}

let reste = 21;
let tour_humain = true;
let humain = [1, 2, 3, 2, 1];
let i = 0;
while reste > 1 {
    let prise = 0;
    if tour_humain {
        prise = humain[i];
        i += 1;
    } else {
        prise = coup_ordi(reste);
    }
    reste -= prise;
    println((tour_humain ? "humain" : "ordi") + " prend " + str(prise) + " -> reste " + str(reste));
    tour_humain = !tour_humain;
}
println(tour_humain ? "l'humain doit prendre le dernier" : "l'ordi doit prendre le dernier");
