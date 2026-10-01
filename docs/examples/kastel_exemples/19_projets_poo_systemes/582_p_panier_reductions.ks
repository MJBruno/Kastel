// ==================================================================
// Exemple 582 — Panier avec codes promo
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : total du panier et réduction en pourcentage selon le code.
// ------------------------------------------------------------------
// Sortie attendue :
//   9.00
//   8.00
//   code inconnu
//   10.00
// ==================================================================

let codes = {"PROMO10": 10, "ETE20": 20};
let articles = [("stylo", 2, 3), ("cahier", 1, 4)];   // (nom, quantité, prix)

func total(code: str) -> float {
    let brut = 0;
    for a in articles { brut += a[1] * a[2]; }
    if code == "" { return brut * 1.0; }
    if !codes.contains(code) {
        println("code inconnu");
        return brut * 1.0;
    }
    return brut * (100 - codes[code]) / 100;
}

println("{:.2f}", total("PROMO10"));
println("{:.2f}", total("ETE20"));
println("{:.2f}", total("XXX"));
