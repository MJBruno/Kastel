// ==================================================================
// Exemple 553 — Facture avec TVA
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : lignes quantité x prix, total HT, TVA et TTC.
// ------------------------------------------------------------------
// Sortie attendue :
//   stylo x2 = 20.00
//   cahier x1 = 5.50
//   HT  25.50
//   TVA 5.10
//   TTC 30.60
// ==================================================================

let lignes = [("stylo", 2, 10.0), ("cahier", 1, 5.5)];
let ht = 0.0;
for l in lignes {
    let total = l[1] * l[2];
    println("{} x{} = {:.2f}", l[0], l[1], total);
    ht += total;
}
let tva = ht * 0.2;
println("HT  {:.2f}", ht);
println("TVA {:.2f}", tva);
println("TTC {:.2f}", ht + tva);
