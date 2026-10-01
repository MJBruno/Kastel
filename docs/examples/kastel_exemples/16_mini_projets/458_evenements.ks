// ==================================================================
// Exemple 458 — Système d'événements
// Catégorie : Mini-projets
// ------------------------------------------------------------------
// Des fonctions abonnées à un nom d'événement.
// ------------------------------------------------------------------
// Sortie attendue :
//   A reçoit bonjour
//   B reçoit bonjour
// ==================================================================

let abonnes = dict();

func s_abonner(evenement: str, f) {
    if !abonnes.contains(evenement) {
        abonnes[evenement] = [];
    }
    abonnes[evenement].add(f);
}

func emettre(evenement: str, donnee) {
    for f in abonnes.get_or(evenement, []) {
        f(donnee);
    }
}

s_abonner("salut", x => println("A reçoit " + x));
s_abonner("salut", x => println("B reçoit " + x));
emettre("salut", "bonjour");
emettre("inconnu", "rien");
