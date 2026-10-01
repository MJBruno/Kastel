// ==================================================================
// Exemple 534 — Machine à pile
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : exécuter un petit programme d'instructions push / add / mul.
// ------------------------------------------------------------------
// Sortie attendue :
//   20
// ==================================================================

let programme = ["push 2", "push 3", "add", "push 4", "mul"];
let pile = [];

for ins in programme {
    let parts = ins.split(" ");
    match parts[0] {
        "push" => { pile.add(parts[1].to_int()); }
        "add" => {
            let b = pile.pop();
            let a = pile.pop();
            pile.add(a + b);
        }
        "mul" => {
            let b = pile.pop();
            let a = pile.pop();
            pile.add(a * b);
        }
        _ => { throw "instruction inconnue : " + parts[0]; }
    }
}
println(pile.pop());
