class Counter {
    function init(value) {
        this.value = value;
    }

    function show() {
        println(this.value);
    }
}

let CounterType = Counter;

let a = new CounterType(10);
let b = new CounterType(20);

a.show();
b.show();