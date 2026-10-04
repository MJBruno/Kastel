// ====================================================================
// Kastel — Fermetures
// Notions : une fonction qui capture et modifie une variable locale
// Résultat attendu :
//   1
//   2
//   3
//   1
// ====================================================================

func creer_compteur() {
    let compte = 0;
    return () => {
        compte = compte + 1;
        return compte;
    };
}

let c1 = creer_compteur();
let c2 = creer_compteur();

println(c1());
println(c1());
println(c1());
println(c2());
