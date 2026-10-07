import std.string;

println(string.capitalize("kastel"));
println(string.slug("Bonjour Kastel !"));
println(string.is_palindrome("kayak"));
println(string.count_occurrences("banana", "an"));

match string.parse_int("42") {
    Ok(value) => println(value + 8);
    Err(error) => println(error);
}

match string.parse_float("3.5") {
    Ok(value) => println(value + 0.5);
    Err(error) => println(error);
}

match string.find("Hello Kastel", "Kastel") {
    Some(index) => println(index);
    None => println("introuvable");
}

match string.pad_left("42", 5, "0") {
    Ok(value) => println(value);
    Err(error) => println(error);
}

match string.truncate("Kastel Programming Language", 10, "...") {
    Ok(value) => println(value);
    Err(error) => println(error);
}

println("std.string: OK");
