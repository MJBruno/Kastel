import std.os;

println(os.platform_name());
println(os.architecture());
println(os.current_directory());
println(os.arguments());
println(os.clock_seconds());

let path = os.environment("PATH");
println(path);

println("std.os: OK");
