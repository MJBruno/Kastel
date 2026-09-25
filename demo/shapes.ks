export class Compteur {
    static let total: int = 12;
    private static let secret: int = 42;

    static func creer() -> Compteur {
        Compteur.total += 1;
        return new Compteur();
    }

    func incrementer() {
        Compteur.total = Compteur.total + 1;
    }
}

