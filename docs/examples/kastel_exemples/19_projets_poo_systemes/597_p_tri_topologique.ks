// ==================================================================
// Exemple 597 — Tri topologique (algorithme de Kahn)
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : ordonner des tâches selon leurs dépendances.
// ------------------------------------------------------------------
// Sortie attendue :
//   ["a", "b", "c", "d"]
// ==================================================================

let graphe = {"a": ["b", "c"], "b": ["d"], "c": ["d"], "d": []};

let entrants = dict();
for s in graphe { entrants[s] = 0; }
for s in graphe {
    for t in graphe[s] {
        entrants[t] = entrants[t] + 1;
    }
}

let file = [];
for s in graphe {
    if entrants[s] == 0 { file.add(s); }
}

let ordre = [];
while !file.is_empty() {
    let s = file.remove_at(0);
    ordre.add(s);
    for t in graphe[s] {
        entrants[t] = entrants[t] - 1;
        if entrants[t] == 0 { file.add(t); }
    }
}
println(ordre);
