// ==================================================================
// Exemple 494 — Bataille navale : tirs
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : une flotte, des tirs, un bilan touché / coulé.
// ------------------------------------------------------------------
// Sortie attendue :
//   0,0 à l'eau
//   1,2 touché
//   1,1 touché
//   4,4 touché
//   9,9 à l'eau
//   1,3 touché
//   flotte coulée : true
// ==================================================================

let bateaux = Set("1,1", "1,2", "1,3", "4,4");
let tirs = ["0,0", "1,2", "1,1", "4,4", "9,9", "1,3"];
let touches = Set();

for t in tirs {
    if bateaux.contains(t) {
        touches.add(t);
        println(t + " touché");
    } else {
        println(t + " à l'eau");
    }
}
println("flotte coulée : " + str(touches.equals(bateaux)));
