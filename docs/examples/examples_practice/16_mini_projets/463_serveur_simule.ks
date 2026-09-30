// ==================================================================
// Exemple 463 — Petit serveur simulé avec tâches
// Catégorie : Mini-projets
// ------------------------------------------------------------------
// Des « requêtes » traitées en parallèle par des travailleurs.
// ------------------------------------------------------------------
// Sortie attendue :
//   requête 1 traitée
//   requête 2 traitée
//   requête 3 traitée
//   requête 4 traitée
// ==================================================================

let requetes = channel<int>();
let reponses = channel<str>();

func travailleur() {
    for i in range(2) {
        let n = requetes.recv();
        reponses.send("requête " + str(n) + " traitée");
    }
}

let w1 = spawn(travailleur);
let w2 = spawn(travailleur);
for n in range(1, 5) {
    requetes.send(n);
}

let recues = [];
for i in range(4) {
    recues.add(reponses.recv());
}
recues.sort();
for r in recues {
    println(r);
}
