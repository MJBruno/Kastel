// ==================================================================
// Exemple 408 — Test de primalité
// Catégorie : Maths et algorithmes
// ------------------------------------------------------------------
// Essayer les diviseurs jusqu'à la racine.
// ------------------------------------------------------------------
// Sortie attendue :
//   [2, 3, 5, 7, 11, 13, 17, 19, 23, 29]
// ==================================================================

func premier(n: int) -> bool {
    if n < 2 {
        return false;
    }
    let d = 2;
    while d * d <= n {
        if n % d == 0 {
            return false;
        }
        d += 1;
    }
    return true;
}

let liste = [];
for n in range(1, 30) {
    if premier(n) {
        liste.add(n);
    }
}
println(liste);
