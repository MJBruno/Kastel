// ==================================================================
// Exemple 616 — Arbre couvrant minimal (Kruskal)
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : trier les arêtes et garder celles qui ne forment pas de cycle.
// ------------------------------------------------------------------
// Sortie attendue :
//   7
// ==================================================================

let aretes = [("A","B",1), ("B","C",2), ("A","C",3), ("C","D",4), ("B","D",5)];

// Tri par poids (insertion).
for i in range(1, aretes.size()) {
    let cle = aretes[i];
    let j = i - 1;
    while j >= 0 && aretes[j][2] > cle[2] {
        aretes[j + 1] = aretes[j];
        j -= 1;
    }
    aretes[j + 1] = cle;
}

let parent = {"A": "A", "B": "B", "C": "C", "D": "D"};

func racine(x: str) -> str {
    while parent[x] != x { x = parent[x]; }
    return x;
}

let poids = 0;
for a in aretes {
    let ra = racine(a[0]);
    let rb = racine(a[1]);
    if ra != rb {
        parent[ra] = rb;
        poids += a[2];
    }
}
println(poids);
