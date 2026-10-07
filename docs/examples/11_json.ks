import std.json;

let data = {
    "name": "Kastel",
    "version": 1,
    "stable": true,
    "features": ["VM", "GC", "HTTP"]
};

match json.encode(data) {
    Ok(text) => {
        println(text);
        match json.decode(text) {
            Ok(value) => {
                println(value);
                match json.get_path(value, "features.1") {
                    Some(feature) => println(feature);
                    None => println("chemin absent");
                }
            }
            Err(error) => println(error);
        }
    }
    Err(error) => println(error);
}

let file = "std_example.json";
match json.write_file(file, data) {
    Ok(_) => println("JSON fichier OK");
    Err(error) => println(error);
}
match json.read_file(file) {
    Ok(value) => println(value);
    Err(error) => println(error);
}

println("std.json: OK");
