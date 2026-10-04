// ====================================================================
// Kastel — Fonction polymorphe
// Notions : paramètre typé par une interface
// Résultat attendu :
//   Woof
//   Miaou
// ====================================================================

interface Animal {
    func cri() -> str;
}

class Chien: Animal {
    func cri() -> str { return "Woof"; }
}

class Chat: Animal {
    func cri() -> str { return "Miaou"; }
}

func faire_parler(a: Animal) {
    println(a.cri());
}

faire_parler(new Chien());
faire_parler(new Chat());
