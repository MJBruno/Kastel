class Animal {
    func init(name) {
        this.name = name;
    }

    func speak() {
        return format("{} makes a sound.", this.name);
    }
}

class Dog : Animal {
    func init(name, breed) {
        base.init(name);
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
