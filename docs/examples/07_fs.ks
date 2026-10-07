import std.fs;

let path = "std_example_fs.txt";

match fs.write_text(path, "ligne 1\nligne 2\n") {
    Ok(_) => println("ecriture OK");
    Err(error) => println(error);
}

match fs.append_text(path, "ligne 3\n") {
    Ok(_) => println("append OK");
    Err(error) => println(error);
}

println(fs.exists(path));

match fs.read_text(path) {
    Ok(content) => println(content);
    Err(error) => println(error);
}

match fs.read_lines(path) {
    Ok(lines) => println(lines);
    Err(error) => println(error);
}

match fs.size_of(path) {
    Ok(size) => println(size);
    Err(error) => println(error);
}

match fs.delete_file(path) {
    Ok(_) => println("suppression OK");
    Err(error) => println(error);
}

println("std.fs: OK");
