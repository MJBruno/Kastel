// À placer dans test/errors/private_field_access.ks
// Doit échouer : `p.age` est interdit hors de la classe Person.

class Person {
    private let age: int = 0;
}

let p = Person();
print(p.age);
