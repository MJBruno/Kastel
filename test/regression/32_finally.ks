func test() {
    try {
        println('try');
        return 7;
    } finally {
        println('finally');
    }
}

println(test());
