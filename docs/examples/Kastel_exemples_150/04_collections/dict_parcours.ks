// ====================================================================
// Kastel — Parcourir un dictionnaire
// Notions : keys, values, entries
// Résultat attendu :
//   maths : 15
//   physique : 12
//   chimie : 18
//   maths = 15
//   total = 45
// ====================================================================

let notes = {"maths": 15, "physique": 12, "chimie": 18};

for matiere in notes.keys() {
    println("{} : {}", matiere, notes[matiere]);
}

for e in notes.entries() {
    println("{} = {}", e[0], e[1]);
    break;
}

let total = 0;
for v in notes.values() {
    total += v;
}
println("total = {}", total);
