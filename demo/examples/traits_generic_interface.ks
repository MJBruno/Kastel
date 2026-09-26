// traits_generic_interface.ks
//
// Une interface "metier" peut elle-meme etendre un contrat intrinseque :
// un peu comme `trait Summable: std::ops::Add<Output = Self> {}` en Rust.
// Une fonction generique contrainte par cette interface peut alors
// utiliser l'operateur directement dans son corps.

interface Addable : Add {
    func add(other: Self) -> Self;
}

class Argent : Addable {
    public let centimes: int;

    func initialize(centimes: int) {
        this.centimes = centimes;
    }

    func add(other: Self) -> Self {
        return new Argent(this.centimes + other.centimes);
    }
}

// `T: Addable` autorise `a + b` A L'INTERIEUR de la fonction generique,
// exactement comme `T: std::ops::Add<Output = T>` en Rust.
func somme_totale<T: Addable>(elements: List<T>) -> T {
    let total: T = elements[0];
    let i = 1;

    while (i < elements.size()) {
        total = total + elements[i];
        i = i + 1;
    }

    return total;
}

let montants: List<Argent> = [new Argent(100), new Argent(250), new Argent(75)];
let total: Argent = somme_totale<Argent>(montants);

print(total.centimes); // 425

// Une interface sans lien avec un operateur ne donne PAS acces a `+` :
// ceci ne compilerait pas si on essayait `a + b` dans le corps.
interface Affichable {
    func afficher() -> str;
}
