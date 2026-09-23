// Classes - methods
class Animal {
    func initialize(name) {
        this.name = name;
    }

    func speak() {
        println(this.name);
    }
}

let animal = new Animal("Milou");
animal.speak();
