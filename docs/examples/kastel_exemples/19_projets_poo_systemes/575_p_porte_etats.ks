// ==================================================================
// Exemple 575 — Machine à états d'une porte
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : un enum pour les états et une fonction de transition.
// ------------------------------------------------------------------
// Sortie attendue :
//   fermer -> fermée
//   verrouiller -> verrouillée
//   ouvrir -> verrouillée
//   deverrouiller -> fermée
//   ouvrir -> ouverte
// ==================================================================

enum Porte {
    Ouverte,
    Fermee,
    Verrouillee
}

func nom(p: Porte) -> str {
    match p {
        Porte.Ouverte => { return "ouverte"; }
        Porte.Fermee => { return "fermée"; }
        Porte.Verrouillee => { return "verrouillée"; }
    }
}

func transition(p: Porte, action: str) -> Porte {
    match p {
        Porte.Ouverte => {
            if action == "fermer" { return Porte.Fermee; }
        }
        Porte.Fermee => {
            if action == "ouvrir" { return Porte.Ouverte; }
            if action == "verrouiller" { return Porte.Verrouillee; }
        }
        Porte.Verrouillee => {
            if action == "deverrouiller" { return Porte.Fermee; }
        }
    }
    return p;          // action impossible : l'état ne change pas
}

let p = Porte.Ouverte;
for a in ["fermer", "verrouiller", "ouvrir", "deverrouiller", "ouvrir"] {
    p = transition(p, a);
    println("{} -> {}", a, nom(p));
}
