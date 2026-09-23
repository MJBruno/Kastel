class Animal {
    func initialize(name) {
        this.name = name;
    }

    func speak() {
        return format("{} makes a sound.", this.name);
    }
}

class Dog : Animal {
    func initialize(name, breed) {
        base.initialize(name);
        this.breed = breed;
    }

    func speak() {
        let base_speech = base.speak();
        return format("{} ({} barks!)", base_speech, this.breed);
    }
}

let generic = new Animal("Creature");
let rex = new Dog("Rex", "Labrador");

println(generic.speak());
println(rex.speak());
println(rex is Animal);
println(rex is Dog);
println(generic is Dog);
