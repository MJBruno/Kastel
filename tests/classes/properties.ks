// Classes - properties
class User {
    func init(name, age) {
        this.name = name;
        this.age = age;
    }
}

let user = new User("Bruno", 25);

println(user.name);
println(user.age);

user.age = 26;
println(user.age);

// Expected error when enabled in a dedicated negative test:
// println(user.missing);
