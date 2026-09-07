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


// MODIFY IN FUNCTION

println("TEST MUTATION TABLEAU");

function modify(values) {
    values[0] = 100;
    values[1] += 10;
}

let numbers = [1, 2, 3];

modify(numbers);

println(numbers[0]);
println(numbers[1]);
println(numbers[2]);