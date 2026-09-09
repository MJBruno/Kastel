class Counter {
    function init(value) {
        this.value = value;
    }

    function increment() {
        this.value = this.value + 1;
    }

    function get() {
        return this.value;
    }
}

let c = new Counter(10);

c.increment();
c.increment();

println(c.get());