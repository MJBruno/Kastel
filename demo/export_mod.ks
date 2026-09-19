export interface Idead {
    func number();
}

export class Personne:Idead {

    func init() {
        println("Constructeur")
    }

    func setAge(age:int) {
        this.age = age
    }

    func getAge()->int {
        return this.age
    }

    func display() {
        println("Age: {}",this.age)
    }

    func number(){
        return 22;
    }

}
