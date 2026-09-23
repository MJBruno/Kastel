// Classes - instance creation
class Animal {
    func initialize(name) {
        this.name = name;
    }
}

let animal = new Animal("Milou");
println(animal.name);
