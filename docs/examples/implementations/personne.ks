// examples/personne.ks
//
// Classe avec champ privé typé et API publique.
// Utilisée par examples/visibility_demo.ks.

export class Personne {

    private let age: int = 0;

    public func initialize(age: int) {
        this.age = age;
    }

    public func setAge(age: int) {
        this.age = age;
    }

    public func getAge() -> int {
        return this.age;
    }

    public func number() -> int {
        return 22;
    }
}
