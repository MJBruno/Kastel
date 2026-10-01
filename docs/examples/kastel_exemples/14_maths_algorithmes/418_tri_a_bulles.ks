// ==================================================================
// Exemple 418 — Tri à bulles
// Catégorie : Maths et algorithmes
// ------------------------------------------------------------------
// Échanger les voisins mal rangés jusqu'à stabilité.
// ------------------------------------------------------------------
// Sortie attendue :
//   [1, 2, 5, 7, 9]
// ==================================================================

func tri_bulles(v: List<int>) {
    let n = v.size();
    for i in range(n) {
        for j in range(n - 1 - i) {
            if v[j] > v[j + 1] {
                let t = v[j];
                v[j] = v[j + 1];
                v[j + 1] = t;
            }
        }
    }
}

let v = [5, 2, 9, 1, 7];
tri_bulles(v);
println(v);
