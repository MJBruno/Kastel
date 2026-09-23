match [1, 2] {
    [1, 2] => println('pair');
    _ => println('other');
}
match 5 {
    x if x > 3 => println('guard');
    _ => println('no');
}
