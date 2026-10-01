// ==================================================================
// Exemple 460 — File d'attente à priorités
// Catégorie : Mini-projets
// ------------------------------------------------------------------
// Insérer chaque élément à sa place pour garder la liste triée.
// ------------------------------------------------------------------
// Sortie attendue :
//   urgente
//   normale
//   basse
// ==================================================================

let file = [];

func inserer(priorite: int, nom: str) {
    let i = 0;
    while i < file.size() && file[i][0] <= priorite {
        i += 1;
    }
    file.insert(i, (priorite, nom));
}

inserer(3, "basse");
inserer(1, "urgente");
inserer(2, "normale");

for e in file {
    println(e[1]);
}
