<<<<<<< HEAD
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
=======
export class ExportMod {
    func init() {
        this.value = this.get_value();
    }


    func get_value() {
        return 55;
>>>>>>> d9a04583b24fbca1c3c82b2fbe5ab774a46c2e4b
    }

}
