function first() {
    println("first");
}

function second() {
    println("second");
}

class Test {
    function callback() {
        println("method");
    }
}

let test = new Test();

let original = test.callback;

test.callback = first;

original();
test.callback();

test.callback = second;

original();
test.callback();