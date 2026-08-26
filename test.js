let x = "global";
function outer() {
    let x = "outer";
    function inner() {
        print(x);
    }
    inner();
}
outer();