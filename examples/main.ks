interface Printable {
    function print();
}

class User : Printable {
    function print() {
        println("user");
    }
}

let u = new User();
u.print();