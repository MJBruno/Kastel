import std.yaml;

let text = "name: Kastel\nversion: 1\nfeatures:\n  - VM\n  - GC\nstable: true\n";

match yaml.parse(text) {
    Ok(value) => println(value);
    Err(error) => println(error);
}

let data = {
    "name": "Kastel",
    "stable": true,
    "version": 1
};

match yaml.stringify(data) {
    Ok(value) => println(value);
    Err(error) => println(error);
}

let file = "std_example.yaml";
match yaml.write_file(file, data) {
    Ok(_) => println("YAML fichier OK");
    Err(error) => println(error);
}
match yaml.read_file(file) {
    Ok(value) => println(value);
    Err(error) => println(error);
}

println("std.yaml: OK");
