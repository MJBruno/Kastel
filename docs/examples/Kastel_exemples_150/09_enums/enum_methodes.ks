// ====================================================================
// Kastel — Énumération avec méthodes
// Notions : func dans un enum, self
// Résultat attendu :
//   Ete est chaud : true
//   Hiver est chaud : false
// ====================================================================

enum Saison {
    Printemps,
    Ete,
    Automne,
    Hiver

    func est_chaude() -> bool {
        return self == Saison.Ete;
    }
}

println("Ete est chaud : {}", Saison.Ete.est_chaude());
println("Hiver est chaud : {}", Saison.Hiver.est_chaude());
