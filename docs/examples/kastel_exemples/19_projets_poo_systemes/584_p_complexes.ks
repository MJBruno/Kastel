// ==================================================================
// Exemple 584 — Nombres complexes
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : partie réelle et imaginaire, addition avec +, multiplication par méthode.
// ------------------------------------------------------------------
// Sortie attendue :
//   4 + 6i
//   -5 + 10i
// ==================================================================

class Complexe : Add {
    let re: int;
    let im: int;

    func initialize(re: int, im: int) {
        self.re = re;
        self.im = im;
    }

    func add(o: Complexe) -> Complexe {
        return new Complexe(self.re + o.re, self.im + o.im);
    }

    func fois(o: Complexe) -> Complexe {
        return new Complexe(self.re * o.re - self.im * o.im, self.re * o.im + self.im * o.re);
    }

    func texte() -> str {
        return str(self.re) + " + " + str(self.im) + "i";
    }
}

let a = new Complexe(1, 2);
let b = new Complexe(3, 4);
println((a + b).texte());
println(a.fois(b).texte());
