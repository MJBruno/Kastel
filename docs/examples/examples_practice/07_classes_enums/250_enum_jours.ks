// ==================================================================
// Exemple 250 — Jours de la semaine
// Catégorie : Classes, interfaces, enums, alias
// ------------------------------------------------------------------
// Un enum comme paramètre de fonction.
// ------------------------------------------------------------------
// Sortie attendue :
//   false
//   true
// ==================================================================

enum Jour {
    Lundi,
    Samedi,
    Dimanche
}

func week_end(j: Jour) -> bool {
    return j == Jour.Samedi || j == Jour.Dimanche;
}

println(week_end(Jour.Lundi));
println(week_end(Jour.Dimanche));
