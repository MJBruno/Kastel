class Animal {
    function speak() {
        println("animal");
    }
}

class Dog : Animal {
    function speak() {
        println("dog");
    }
}

let d = new Dog();

d.speak();