import std.statistics;

let values = [1.0, 2.0, 3.0, 4.0, 5.0];

match statistics.mean(values) {
    Some(value) => println(value);
    None => println("mean absent");
}

match statistics.median(values) {
    Some(value) => println(value);
    None => println("median absent");
}

match statistics.variance_population(values) {
    Some(value) => println(value);
    None => println("variance absent");
}

match statistics.stdev_population(values) {
    Some(value) => println(value);
    None => println("stdev absent");
}

println(statistics.range_of(values));

match statistics.correlation(values, [2.0, 4.0, 6.0, 8.0, 10.0]) {
    Ok(value) => println(value);
    Err(error) => println(error);
}

println("std.statistics: OK");
