// ==================================================================
// Exemple 461 — Parcours en largeur d'un graphe
// Catégorie : Mini-projets
// ------------------------------------------------------------------
// Une file, un ensemble de sommets visités.
// ------------------------------------------------------------------
// Sortie attendue :
//   ["A", "B", "C", "D"]
// ==================================================================

let graphe = {
    "A": ["B", "C"],
    "B": ["D"],
    "C": ["D"],
    "D": []
};

let visites = Set("A");
let file = ["A"];
let ordre = [];

while !file.is_empty() {
    let courant = file.remove_at(0);
    ordre.add(courant);
    for voisin in graphe[courant] {
        if !visites.contains(voisin) {
            visites.add(voisin);
            file.add(voisin);
        }
    }
}
println(ordre);
