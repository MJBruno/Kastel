// ==================================================================
// Exemple 637 — Nombres d'Armstrong
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : les nombres à 3 chiffres égaux à la somme des cubes de leurs chiffres.
// ------------------------------------------------------------------
// Sortie attendue :
//   [153, 370, 371, 407]
// ==================================================================

let res = [];
for n in range(100, 1000) {
    let a = idiv(n, 100);
    let b = idiv(n, 10) % 10;
    let c = n % 10;
    if a * a * a + b * b * b + c * c * c == n {
        res.add(n);
    }
}
println(res);
