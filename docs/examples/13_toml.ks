import std.toml;

let text = "name = \"Kastel\"\nversion = 1\nstable = true";

match toml.parse(text) {
    Ok(value) => {
        println(value);
        println(value.get("name"));
    }
    Err(error) => println(error);
}

let data = {
    "name": "Kastel",
    "version": 1,
    "stable": true
};

match toml.stringify(data) {
    Ok(value) => println(value);
    Err(error) => println(error);
}

let file = "std_example.toml";
match toml.write_file(file, data) {
    Ok(_) => println("TOML fichier OK");
    Err(error) => println(error);
}
match toml.read_file(file) {
    Ok(value) => println(value);
    Err(error) => println(error);
}

println("std.toml: OK");
