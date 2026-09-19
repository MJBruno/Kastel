 
export interface Idead {
    func number()->int;
}

export class Personne:Idead {

    func init() {
        println("Constructeur")
    }
 
    func init(age) {
        this.age = age
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

    func number()->int{
        return 22;
<<<<<<< HEAD
=======
 
>>>>>>> 612fd4675a3c80d24c1c910a09531c022981e183
    }
}
