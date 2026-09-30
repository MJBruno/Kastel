// ==================================================================
// Exemple 249 — Enum avec méthode
// Catégorie : Classes, interfaces, enums, alias
// ------------------------------------------------------------------
// Un enum peut contenir des méthodes qui utilisent self.
// ------------------------------------------------------------------
// Sortie attendue :
//   false
//   true
// ==================================================================

enum Statut {
    EnCours,
    Termine

    func est_fini() -> bool {
        return self == Statut.Termine;
    }
}

println(Statut.EnCours.est_fini());
println(Statut.Termine.est_fini());
