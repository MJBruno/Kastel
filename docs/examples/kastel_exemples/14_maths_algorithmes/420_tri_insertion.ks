// ==================================================================
// Exemple 420 — Tri par insertion
// Catégorie : Maths et algorithmes
// ------------------------------------------------------------------
// Insérer chaque élément à sa place dans la partie déjà triée.
// ------------------------------------------------------------------
// Sortie attendue :
//   [5, 6, 11, 12, 13]
// ==================================================================

func tri_insertion(v: List<int>) {
    for i in range(1, v.size()) {
        let cle = v[i];
        let j = i - 1;
        while j >= 0 && v[j] > cle {
            v[j + 1] = v[j];
            j -= 1;
        }
        v[j + 1] = cle;
    }
}

let v = [12, 11, 13, 5, 6];
tri_insertion(v);
println(v);
