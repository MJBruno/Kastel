 
export class Animal {
    func speak() {
        println("animal");
    }
}

export class Dog : Animal {
    func init(name) {
        this.name = name
    }
    func speak() {
        println("Je suis {}",this.name);
    }
}


export func details() {
    println("Terminer")
};