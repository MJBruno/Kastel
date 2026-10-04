// ====================================================================
// Kastel — Méthodes privées
// Notions : private func, usage interne uniquement
// Résultat attendu :
//   Dossier valide
//   Dossier invalide
// ====================================================================

class Dossier {
    let pieces: int;

    func initialize(pieces: int) {
        self.pieces = pieces;
    }

    private func est_complet() -> bool {
        return self.pieces >= 3;
    }

    func statut() -> str {
        return self.est_complet() ? "Dossier valide" : "Dossier invalide";
    }
}

println(new Dossier(4).statut());
println(new Dossier(1).statut());
