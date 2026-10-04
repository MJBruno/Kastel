// ====================================================================
// Kastel — Insérer et retirer
// Notions : insert, remove (par valeur), remove_at (par position), pop
// Résultat attendu :
//   [10, 15, 20, 30]
//   true
//   10
//   [15, 30]
//   30
//   [15]
// ====================================================================

let nombres = [10, 20, 30];

nombres.insert(1, 15);
println(nombres);

let present = nombres.remove(20);   // retire par valeur : true si trouvé
println(present);

let retire = nombres.remove_at(0);  // retire par position et renvoie l'élément
println(retire);
println(nombres);

let dernier = nombres.pop();
println(dernier);
println(nombres);
