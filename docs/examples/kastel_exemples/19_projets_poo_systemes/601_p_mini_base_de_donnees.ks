// ==================================================================
// Exemple 601 — Mini base de données
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : filtrer, projeter et regrouper des enregistrements.
// ------------------------------------------------------------------
// Sortie attendue :
//   Ada, Alan
//   Paris : 2
// ==================================================================

let table = [
    { nom: "Ada", age: 36, ville: "Paris" },
    { nom: "Alan", age: 41, ville: "Londres" },
    { nom: "Grace", age: 29, ville: "Paris" }
];

// SELECT nom WHERE age > 30
let noms = [];
for ligne in table {
    if ligne.age > 30 {
        noms.add(ligne.nom);
    }
}
println(", ".join(noms));

// GROUP BY ville, COUNT(*)
let par_ville = dict();
for ligne in table {
    par_ville[ligne.ville] = par_ville.get_or(ligne.ville, 0) + 1;
}
println("Paris : {}", par_ville["Paris"]);
