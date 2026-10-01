// ==================================================================
// Exemple 607 — Réservation de salle
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : refuser les créneaux qui chevauchent une réservation existante.
// ------------------------------------------------------------------
// Sortie attendue :
//   true
//   false
//   true
//   true
// ==================================================================

let reservations = [];

func reserver(debut: int, fin: int) -> bool {
    for r in reservations {
        if r[0] < fin && debut < r[1] {
            return false;        // chevauchement
        }
    }
    reservations.add((debut, fin));
    return true;
}

println(reserver(9, 11));
println(reserver(10, 12));
println(reserver(11, 12));
println(reserver(8, 9));
