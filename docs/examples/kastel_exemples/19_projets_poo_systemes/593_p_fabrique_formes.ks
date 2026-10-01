// ==================================================================
// Exemple 593 — Patron Fabrique
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : créer un objet à partir d'un nom, en signalant les noms inconnus.
// ------------------------------------------------------------------
// Sortie attendue :
//   16
//   forme inconnue : cercle
// ==================================================================

interface Forme {
    func aire() -> int;
}

class Carre: Forme {
    let c: int;
    func initialize(c: int) { self.c = c; }
    func aire() -> int { return self.c * self.c; }
}

class Rectangle: Forme {
    let l: int;
    let h: int;
    func initialize(l: int, h: int) { self.l = l; self.h = h; }
    func aire() -> int { return self.l * self.h; }
}

func fabriquer(nom: str, a: int, b: int) -> Result<Forme, str> {
    if nom == "carre" { return Ok(new Carre(a)); }
    if nom == "rectangle" { return Ok(new Rectangle(a, b)); }
    return Err("forme inconnue : " + nom);
}

match fabriquer("carre", 4, 0) {
    Ok(f) => { println(f.aire()); }
    Err(e) => { println(e); }
}
match fabriquer("cercle", 1, 0) {
    Ok(f) => { println(f.aire()); }
    Err(e) => { println(e); }
}
