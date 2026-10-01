// ==================================================================
// Exemple 493 — Labyrinthe : existe-t-il un chemin ?
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : parcours en profondeur avec un ensemble de cases visitées.
// ------------------------------------------------------------------
// Sortie attendue :
//   true
// ==================================================================

let lab = [
    "S.#.",
    ".##.",
    "...E"
];
let visites = Set();

func chercher(x: int, y: int) -> bool {
    if x < 0 || x >= 4 || y < 0 || y >= 3 { return false; }
    let c = lab[y].char_at(x);
    let cle = str(x) + "," + str(y);
    if c == "#" || visites.contains(cle) { return false; }
    if c == "E" { return true; }
    visites.add(cle);
    return chercher(x + 1, y) || chercher(x - 1, y) || chercher(x, y + 1) || chercher(x, y - 1);
}

println(chercher(0, 0));
