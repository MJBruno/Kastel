println("TEST UPVALUES NIVEAU 2");

function outer() {
    let x = 10;

    function middle() {
        function inner() {
            return x;
        }

        return inner;
    }

    return middle();
}

let fn = outer();

println(fn());