import std.process;

match process.run("git", ["--version"]) {
    Ok(output) => {
        println(output.stdout());
        println(output.stderr());
        println(output.code());
        println(output.success());
    }
    Err(error) => println(error);
}

match process.run0("git") {
    Ok(output) => println(output.code());
    Err(error) => println(error);
}

println("std.process: OK");
