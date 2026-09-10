function test() {
    try {
        return 10;
    } finally {
        println("finally");
    }
}

println(test());