// ==================================================================
// Exemple 588 — Convertisseur avec méthodes static
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : une classe utilitaire sans instance.
// ------------------------------------------------------------------
// Sortie attendue :
//   212.0
//   0.0
//   100.0
// ==================================================================

class Temp {
    static func c_vers_f(c: float) -> float {
        return c * 9.0 / 5.0 + 32.0;
    }

    static func f_vers_c(f: float) -> float {
        return (f - 32.0) * 5.0 / 9.0;
    }
}

println(Temp.c_vers_f(100.0));
println(Temp.f_vers_c(32.0));
println(Temp.f_vers_c(212.0));
