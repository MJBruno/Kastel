// Classes - methods
class Animal {
    func init(name) {
        this.name = name;
    }

    func speak() {
        println(this.name);
    }
}

let animal = new Animal("Milou");
animal.speak();
