function outer() {
    const x = 10;

    function inner() {
        println(x);
    }

    inner();
}

outer();