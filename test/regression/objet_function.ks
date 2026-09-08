println("TEST OBJET + FONCTION");

function birthday(user) {
    user.age += 1;
}

let user = {
    name: "Bruno",
    age: 30
};

birthday(user);

println(user.name);
println(user.age);