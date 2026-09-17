// std/math.ks
//
// Compléments mathématiques au-dessus des fonctions natives
// (abs, floor, sqrt, pow, min, max, ...).
//
// Usage : from std.math import clamp, gcd, sum;

export func clamp(value, low, high) {
    if value < low {
        return low;
    }

    if value > high {
        return high;
    }

    return value;
}

export func lerp(start, stop, t) {
    return start + (stop - start) * t;
}

export func gcd(a, b) {
    let x = abs(a);
    let y = abs(b);

    while y != 0 {
        let remainder = x % y;
        x = y;
        y = remainder;
    }

    return x;
}

export func lcm(a, b) {
    if a == 0 || b == 0 {
        return 0;
    }

    // `/` renvoie toujours un flottant en Kastel : floor() ramène le
    // résultat vers l'entier attendu pour un "plus petit multiple
    // commun".
    return floor(abs(a * b) / gcd(a, b));
}

export func sum(items) {
    let total = 0;

    for item in items {
        total = total + item;
    }

    return total;
}

export func average(items) {
    if items.length == 0 {
        return 0;
    }

    return sum(items) / items.length;
}

export func min_of(items) {
    let result = items.get(0);

    for item in items {
        result = min(result, item);
    }

    return result;
}

export func max_of(items) {
    let result = items.get(0);

    for item in items {
        result = max(result, item);
    }

    return result;
}
