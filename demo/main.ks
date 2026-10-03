class Compteur {
    static let total: int = 0;
    private static let secret: int = 42;

    static func creer() -> Compteur {
        Compteur.total += 1;
        return new Compteur();
    }
    
    func incrementer() {
        Compteur.total += 1;
    }
}

let c = Compteur.creer();
c.incrementer();
c.incrementer();
print(Compteur.total);   // 2