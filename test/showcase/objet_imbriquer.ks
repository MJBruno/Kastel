println("TEST OBJETS IMBRIQUES");

let user = {
    name: "Bruno",
    address: {
        city: "Antananarivo",
        country: "Madagascar"
    }
};

println(user.address.city);
println(user.address.country);

user.address.city = "Toamasina";

println(user.address.city);