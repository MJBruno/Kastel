class User {
    func init(name, age) {
        this.name = name;
        this.age = age;
    }

    func greet() {
        return 'Hello ' + this.name;
    }
}

let user = new User('Bruno', 30);
println(user.name);
println(user.age);
println(user.greet());
println(type(user));
