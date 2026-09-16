match 3 {
    1 .. 3 => println('exclusive');
    _ => println('other');
}
match 3 {
    1 ..= 3 => println('inclusive');
    _ => println('other');
}
