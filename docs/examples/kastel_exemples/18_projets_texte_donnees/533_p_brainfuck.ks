// ==================================================================
// Exemple 533 — Interpréteur Brainfuck
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : un langage minimal à 8 instructions ; ce programme place 65 dans la cellule 1.
// ------------------------------------------------------------------
// Sortie attendue :
//   65
// ==================================================================

let prog = "++++++++[>++++++++<-]>+.";

// Pré-calcul des crochets correspondants.
let saut = dict();
let pile = [];
for i in range(prog.size()) {
    let c = prog.char_at(i);
    if c == "[" { pile.add(i); }
    if c == "]" {
        let j = pile.pop();
        saut[str(i)] = j;
        saut[str(j)] = i;
    }
}

let bande = [];
for i in range(10) { bande.add(0); }
let ptr = 0;
let pc = 0;

while pc < prog.size() {
    let c = prog.char_at(pc);
    if c == "+" { bande[ptr] += 1; }
    if c == "-" { bande[ptr] -= 1; }
    if c == ">" { ptr += 1; }
    if c == "<" { ptr -= 1; }
    if c == "." { println(bande[ptr]); }
    if c == "[" && bande[ptr] == 0 { pc = saut[str(pc)]; }
    if c == "]" && bande[ptr] != 0 { pc = saut[str(pc)]; }
    pc += 1;
}
