// ====================================================================
// Kastel — Liste de listes
// Notions : indexation imbriquée m[i][j]
// Résultat attendu :
//   4
//   [1, 2, 30]
//   somme = 51
// ====================================================================

let m = [[1, 2, 3], [4, 5, 6]];

println(m[1][0]);

m[0][2] = 30;
println(m[0]);

let somme = 0;
for ligne in m {
    for v in ligne {
        somme += v;
    }
}
println("somme = {}", somme);
