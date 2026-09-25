class Animal {
    protected let name: str = "Animal";

    protected func speak() -> str {
        return "sound";
    }
}

class Dog: Animal {
    func describe() -> str {
        return this.name + " " + this.speak();
    }
}

let dog = new Dog();
println(dog.describe());

// erreur
// println(dog.name);