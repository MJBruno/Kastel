// ====================================================================
// Kastel — Tester le type d'un objet
// Notions : is avec une classe ou une interface
// Résultat attendu :
//   true
//   false
//   true
//   false
// ====================================================================

interface Volant {
    func voler() -> str;
}

class Oiseau: Volant {
    func voler() -> str { return "bat des ailes"; }
}

class Poisson {
    func nager() -> str { return "nage"; }
}

let a = new Oiseau();
let b = new Poisson();

println(a is Oiseau);
println(a is Poisson);
println(a is Volant);
println(b is Volant);
