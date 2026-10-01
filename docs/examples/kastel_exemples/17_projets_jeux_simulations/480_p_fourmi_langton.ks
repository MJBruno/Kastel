// ==================================================================
// Exemple 480 — Fourmi de Langton
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : une fourmi tourne selon la couleur de la case ; on compte les cases noires après 100 pas (grille torique 21 x 21).
// ==================================================================

let N = 21;
let cases = [];
for i in range(N * N) { cases.add(0); }

let x = 10;
let y = 10;
let dir = 0;                       // 0 haut, 1 droite, 2 bas, 3 gauche
let dx = [0, 1, 0, -1];
let dy = [-1, 0, 1, 0];

for pas in range(100) {
    let idx = y * N + x;
    if cases[idx] == 0 {
        dir = (dir + 1) % 4;       // case blanche : tourner à droite
        cases[idx] = 1;
    } else {
        dir = (dir + 3) % 4;       // case noire : tourner à gauche
        cases[idx] = 0;
    }
    x = (x + dx[dir] + N) % N;
    y = (y + dy[dir] + N) % N;
}

let noires = 0;
for c in cases { noires += c; }
println(noires);
