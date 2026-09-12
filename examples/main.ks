let values = [1, 2, 3];

println(values.map(function(x) {
    return [x, x + 1].map(function(y) {
        return y * 10;
    });
})); 