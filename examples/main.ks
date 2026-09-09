function test() {
    try {
        throw "error";
    } finally {
        println("cleanup");
        return 20;
    }
}

println(test());