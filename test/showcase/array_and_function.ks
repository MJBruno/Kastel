println("TEST FONCTION + TABLEAU");

function sum(values) {
    let total = 0;

    for value in values {
        total += value;
    }

    return total;
}

let numbers = [10, 20, 30, 40];

println(sum(numbers));


