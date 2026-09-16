// Classes - inheritance
class Animal {
    func speak() {
        println("animal");
    }
}

class Dog : Animal {
    func speak() {
        println("dog");
    }
}

let animal = new Animal();
let dog = new Dog();

animal.speak();
dog.speak();
