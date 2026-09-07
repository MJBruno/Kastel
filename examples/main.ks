println("TEST FONCTIONS IMBRIQUEES");

function outer() {
    function inner() {
        return 42;
    }

    return inner();
}

println(outer());