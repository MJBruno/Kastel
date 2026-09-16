interface Speakable {
    func speak();
}

class Dog: Speakable {
    func speak() {
        return 'woof';
    }
}

let d = new Dog();
println(d.speak());
println(d is Speakable);
