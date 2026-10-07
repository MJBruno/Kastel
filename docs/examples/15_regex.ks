import std.regex;

match regex.is_match("^[A-Z][a-z]+$", "Kastel") {
    Ok(value) => println(value);
    Err(error) => println(error);
}

match regex.find("(Kastel) ([0-9]+)", "Kastel 2026") {
    Ok(value) => {
        match value {
            Some(result) => {
                println(result.start);
                println(result.end);
                println(result.groups);
            }
            None => println("aucune correspondance");
        }
    }
    Err(error) => println(error);
}

match regex.find_all("\\d+", "2026 10 07") {
    Ok(value) => println(value);
    Err(error) => println(error);
}

match regex.replace("([a-z]+)-([0-9]+)", "kastel-42", "$2:$1") {
    Ok(value) => println(value);
    Err(error) => println(error);
}

match regex.split("\\s+", "Kastel langage Rust") {
    Ok(value) => println(value);
    Err(error) => println(error);
}

println(regex.escape("a+b*c"));
println("std.regex: OK");
