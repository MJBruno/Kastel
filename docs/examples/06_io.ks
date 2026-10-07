import std.io;

io.println_value("std.io demonstration");
io.print_value("Votre nom : ");
let name = io.read_line();
io.println_value("Bonjour " + name);

let language = io.read_line_with_prompt("Langage prefere : ");
io.println_value("Langage = " + language);

println("std.io: OK");
