// ==================================================================
// Exemple 419 — Tri par sélection
// Catégorie : Maths et algorithmes
// ------------------------------------------------------------------
// Chercher le minimum et le placer en tête, en répétant.
// ------------------------------------------------------------------
// Sortie attendue :
//   [11, 12, 22, 25, 64]
// ==================================================================

func tri_selection(v: List<int>) {
    for i in range(v.size()) {
        let m = i;
        for j in range(i + 1, v.size()) {
            if v[j] < v[m] {
                m = j;
            }
        }
        let t = v[i];
        v[i] = v[m];
        v[m] = t;
    }
}

let v = [64, 25, 12, 22, 11];
tri_selection(v);
println(v);
