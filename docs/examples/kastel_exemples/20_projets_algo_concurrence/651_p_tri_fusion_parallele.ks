// ==================================================================
// Exemple 651 — Tri fusion parallèle
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : deux tâches trient chacune une moitié, puis on fusionne.
// ------------------------------------------------------------------
// Sortie attendue :
//   [1, 2, 3, 4, 5, 6, 7, 8]
// ==================================================================

func trier(v: List<int>) -> List<int> {
    let c = v.copy();
    c.sort();
    return c;
}

func fusion(a: List<int>, b: List<int>) -> List<int> {
    let res = [];
    let i = 0;
    let j = 0;
    while i < a.size() && j < b.size() {
        if a[i] <= b[j] { res.add(a[i]); i += 1; }
        else { res.add(b[j]); j += 1; }
    }
    while i < a.size() { res.add(a[i]); i += 1; }
    while j < b.size() { res.add(b[j]); j += 1; }
    return res;
}

let donnees = [8, 3, 5, 1, 7, 2, 6, 4];
let t1 = spawn(trier, donnees.slice(0, 4));
let t2 = spawn(trier, donnees.slice(4, 8));
println(fusion(t1.join(), t2.join()));
