import std.math;

match math.safe_sqrt(144.0) {
    Ok(value) => println(value);
    Err(error) => println(error);
}

match math.safe_log(10.0) {
    Ok(value) => println(value);
    Err(error) => println(error);
}

println(math.gcd(48, 18));
println(math.lcm(12, 18));
println(math.is_prime(97));
println(math.lerp(0.0, 10.0, 0.25));
println(math.to_degrees(3.141592653589793));

match math.average([10.0, 20.0, 30.0]) {
    Some(value) => println(value);
    None => println("aucune valeur");
}

println(math.min_of(7, 3));
println(math.max_of(7, 3));
println(math.clamp(15, 0, 10));

println("std.math: OK");
