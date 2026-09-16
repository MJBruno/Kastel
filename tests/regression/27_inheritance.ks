class Animal {
    func init(name) {
        this.name = name;
    }

    func speak() {
        return this.name + ': animal';
    }
}

class Dog: Animal {
    func speak() {
        return base.speak() + ' dog';
    }
}

let d = new Dog('Milou');
println(d.speak());
println(d is Dog);
println(d is Animal);
