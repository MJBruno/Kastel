// ==================================================================
// Exemple 644 — MapReduce : compter des mots
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : chaque tâche compte un texte (map), puis on fusionne les comptes (reduce).
// ------------------------------------------------------------------
// Sortie attendue :
//   a=3 b=2 c=1
// ==================================================================

func compter(texte: str) -> Dict<str, int> {
    let d = dict();
    for m in texte.split(" ") {
        d[m] = d.get_or(m, 0) + 1;
    }
    return d;
}

let t1 = spawn(compter, "a b a");
let t2 = spawn(compter, "b c a");

let total = dict();
for d in [t1.join(), t2.join()] {
    for m in d {
        total[m] = total.get_or(m, 0) + d[m];
    }
}
println("a={} b={} c={}", total["a"], total["b"], total["c"]);
