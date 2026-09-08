println("TEST NULL + TABLEAU");

let values = [null, 10, null, "Kastel"];

println(values[0]);
println(values[1]);
println(values[2]);
println(values[3]);

for value in values {
    if value == null {
        println("NULL");
    } else {
        println("VALUE");
    }
}