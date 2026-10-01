// ==================================================================
// Exemple 614 — Plus courts chemins de Dijkstra
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : distances depuis un sommet dans un graphe pondéré.
// ------------------------------------------------------------------
// Sortie attendue :
//   A 0
//   B 7
//   C 9
//   D 20
//   E 20
//   F 11
// ==================================================================

let aretes = [("A","B",7), ("A","C",9), ("A","F",14), ("B","C",10), ("B","D",15),
              ("C","D",11), ("C","F",2), ("D","E",6), ("E","F",9)];
let sommets = ["A", "B", "C", "D", "E", "F"];

let voisins = dict();
for s in sommets { voisins[s] = []; }
for a in aretes {
    voisins[a[0]].add((a[1], a[2]));
    voisins[a[1]].add((a[0], a[2]));
}

let INF = 1000000;
let dist = dict();
for s in sommets { dist[s] = INF; }
dist["A"] = 0;
let vus = Set();

for tour in range(sommets.size()) {
    // Sommet non visité le plus proche.
    let courant = "";
    for s in sommets {
        if !vus.contains(s) && (courant == "" || dist[s] < dist[courant]) {
            courant = s;
        }
    }
    vus.add(courant);
    for v in voisins[courant] {
        let d = dist[courant] + v[1];
        if d < dist[v[0]] {
            dist[v[0]] = d;
        }
    }
}

for s in sommets {
    println("{} {}", s, dist[s]);
}
