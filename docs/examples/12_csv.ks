import std.csv;

let text = "name,age\nBruno,29\nAlice,31\n";

match csv.parse_default(text) {
    Ok(rows) => println(rows);
    Err(error) => println(error);
}

match csv.parse_with_header(text, ",") {
    Ok(rows) => println(rows);
    Err(error) => println(error);
}

let rows = [
    ["Bruno", 29],
    ["Alice", 31]
];

match csv.stringify(rows, ",") {
    Ok(value) => println(value);
    Err(error) => println(error);
}

let file = "std_example.csv";
match csv.write_file(file, rows, ",") {
    Ok(_) => println("CSV fichier OK");
    Err(error) => println(error);
}
match csv.read_file(file, ",") {
    Ok(value) => println(value);
    Err(error) => println(error);
}

println("std.csv: OK");
