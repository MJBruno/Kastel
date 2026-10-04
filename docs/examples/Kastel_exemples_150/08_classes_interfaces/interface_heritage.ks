// ====================================================================
// Kastel — Interface qui étend une autre
// Notions : interface B: A
// Résultat attendu :
//   Rex est un animal
//   Rex aboie
// ====================================================================

interface Animal {
    func nom() -> str;
}

interface Chien: Animal {
    func aboyer() -> str;
}

class Labrador: Chien {
    func nom() -> str { return "Rex"; }
    func aboyer() -> str { return "Rex aboie"; }
}

let c: Chien = new Labrador();
println("{} est un animal", c.nom());
println(c.aboyer());
