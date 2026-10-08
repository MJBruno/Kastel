import std.regex

let matched = regex.is_match("^(hello|bonjour) [a-z]+$", "bonjour kastel");
match matched {
    Ok(true) => println("regex is_match: ok");
    _ => println("regex is_match: failed");
}

let found = regex.find("(kastel)-([0-9]+)", "langage kastel-42");
match found {
    Ok(Some(value)) => println(value.text + " @ " + str(value.start));
    _ => println("regex find: failed");
}

let replaced = regex.replace("([a-z]+)-([0-9]+)", "kastel-42", "$2:$1");
match replaced {
    Ok(value) => println(value);
    _ => println("regex replace: failed");
}

let split = regex.split(",\\s*", "a, b,c");
match split {
    Ok(value) => println(value);
    _ => println("regex split: failed");
}

println("std.regex smoke tests: ok");
