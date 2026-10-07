import std.path;

let file_path = path.join(["src", "compiler", "types.rs"]);
println(file_path);
println(path.basename(file_path));
println(path.dirname(file_path));

match path.extension(file_path) {
    Some(value) => println(value);
    None => println("pas d'extension");
}

match path.stem(file_path) {
    Some(value) => println(value);
    None => println("pas de stem");
}

match path.absolute(".") {
    Ok(value) => println(value);
    Err(error) => println(error);
}

println(path.is_file(file_path));
println("std.path: OK");
