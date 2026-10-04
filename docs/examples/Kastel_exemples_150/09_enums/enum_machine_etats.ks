// ====================================================================
// Kastel — Machine à états
// Notions : enum + fonction de transition
// Résultat attendu :
//   Attente -> Marche -> Pause -> Marche -> Arret
// ====================================================================

enum Etat {
    Attente,
    Marche,
    Pause,
    Arret
}

func nom(e: Etat) -> str {
    match e {
        Etat.Attente => { return "Attente"; },
        Etat.Marche => { return "Marche"; },
        Etat.Pause => { return "Pause"; },
        Etat.Arret => { return "Arret"; }
    }
}

func suivant(e: Etat, evenement: str) -> Etat {
    if e == Etat.Attente && evenement == "start" {
        return Etat.Marche;
    }
    if e == Etat.Marche && evenement == "pause" {
        return Etat.Pause;
    }
    if e == Etat.Pause && evenement == "start" {
        return Etat.Marche;
    }
    if e == Etat.Marche && evenement == "stop" {
        return Etat.Arret;
    }
    return e;
}

let etat = Etat.Attente;
let trace = [nom(etat)];

for ev in ["start", "pause", "start", "stop"] {
    etat = suivant(etat, ev);
    trace.add(nom(etat));
}

println(trace.join(" -> "));
