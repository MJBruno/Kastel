// ==================================================================
// Exemple 412 — Nombres parfaits
// Catégorie : Maths et algorithmes
// ------------------------------------------------------------------
// Égaux à la somme de leurs diviseurs propres.
// ------------------------------------------------------------------
// Sortie attendue :
//   6
//   28
//   496
// ==================================================================

func parfait(n: int) -> bool {
    let somme = 0;
    for d in range(1, n) {
        if n % d == 0 {
            somme += d;
        }
    }
    return somme == n;
}

for n in range(2, 500) {
    if parfait(n) {
        println(n);
    }
}
