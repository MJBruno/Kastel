 
export class Animal {
    func init() {
        println("constructeur animal");
    }
    func speak() {
        println("animal");
    }
}

export class Dog : Animal {
    func init(name) {
        this.name = name
    }

    
}


export func details() {
    println("Terminer")
}