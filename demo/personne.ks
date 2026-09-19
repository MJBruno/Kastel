// examples/personne.ks
//
// Classe avec champ privé typé et API publique.
// Utilisée par examples/visibility_demo.ks.

export class Personne {

    public let age: int = 0;

    func initialize(age: int) {
        this.age = age;
    }

    func setAge(age: int) {
        this.age = age;
    }

    func getAge() -> int {
        return this.age;
    }

    func number() -> int {
        return 22;
    }
}
