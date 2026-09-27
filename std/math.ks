// std/math.ks
//
// Les fonctions natives (sqrt, log, pow, ...) ne renvoient jamais
// d'erreur Kastel à proprement parler : hors domaine, elles renvoient
// silencieusement NaN/Inf (sqrt, log, asin, acos) ou lèvent une
// exception native (pow en cas de débordement entier). Ce module les
// enveloppe en Result<T, str> pour un usage idiomatique avec `match`,
// et ajoute des utilitaires génériques appuyés sur les capabilities
// Ord/Add des types primitifs (int, float, str).

// ------------------------------------------------------------------
// Variantes sûres des fonctions à domaine restreint
// ------------------------------------------------------------------

export func safe_sqrt(value: float) -> Result<float, str> {
    if value < 0.0 {
        return Err("safe_sqrt: argument negatif (" + str(value) + ")");
    }
    return Ok(sqrt(value));
}

export func safe_log(value: float) -> Result<float, str> {
    if value <= 0.0 {
        return Err("safe_log: argument non positif (" + str(value) + ")");
    }
    return Ok(log(value));
}

export func safe_log10(value: float) -> Result<float, str> {
    if value <= 0.0 {
        return Err("safe_log10: argument non positif (" + str(value) + ")");
    }
    return Ok(log10(value));
}

export func safe_asin(value: float) -> Result<float, str> {
    if value < -1.0 || value > 1.0 {
        return Err("safe_asin: argument hors de [-1, 1] (" + str(value) + ")");
    }
    return Ok(asin(value));
}

export func safe_acos(value: float) -> Result<float, str> {
    if value < -1.0 || value > 1.0 {
        return Err("safe_acos: argument hors de [-1, 1] (" + str(value) + ")");
    }
    return Ok(acos(value));
}

export func safe_div(a: float, b: float) -> Result<float, str> {
    if b == 0.0 {
        return Err("safe_div: division par zero");
    }
    return Ok(a / b);
}

export func safe_idiv(a: int, b: int) -> Result<int, str> {
    if b == 0 {
        return Err("safe_idiv: division par zero");
    }
    return Ok(idiv(a, b));
}

// pow() natif lève une exception native (débordement i64) plutôt que
// de renvoyer un Result : on la capture ici pour rester dans le monde
// Result plutôt que de forcer l'appelant à faire un try/catch.
export func safe_pow(base: int, exponent: int) -> Result<int, str> {
    try {
        return Ok(pow(base, exponent));
    } catch (error) {
        return Err(error);
    }
}

// ------------------------------------------------------------------
// Utilitaires génériques (Ord / Add)
// ------------------------------------------------------------------

export func clamp<T: Ord>(value: T, low: T, high: T) -> T {
    if value < low {
        return low;
    }
    if value > high {
        return high;
    }
    return value;
}

export func min_of<T: Ord>(a: T, b: T) -> T {
    if a < b {
        return a;
    }
    return b;
}

export func max_of<T: Ord>(a: T, b: T) -> T {
    if a > b {
        return a;
    }
    return b;
}

export func min_list<T: Ord>(values: List<T>) -> Option<T> {
    if values.is_empty() {
        return None;
    }

    let smallest = values.first();
    for value in values {
        if value < smallest {
            smallest = value;
        }
    }
    return Some(smallest);
}

export func max_list<T: Ord>(values: List<T>) -> Option<T> {
    if values.is_empty() {
        return None;
    }

    let largest = values.first();
    for value in values {
        if value > largest {
            largest = value;
        }
    }
    return Some(largest);
}

// `zero` est l'élément neutre fourni par l'appelant (0, 0.0, "") : Add
// ne garantit pas d'élément neutre générique, donc pas de somme d'une
// liste vide sans lui.
export func sum<T: Add>(values: List<T>, zero: T) -> T {
    let total = zero;
    for value in values {
        total = total + value;
    }
    return total;
}

export func average(values: List<float>) -> Option<float> {
    if values.is_empty() {
        return None;
    }
    return Some(sum(values, 0.0) / values.size());
}

// ------------------------------------------------------------------
// Arithmétique entière
// ------------------------------------------------------------------

export func gcd(a: int, b: int) -> int {
    let x = abs(a);
    let y = abs(b);
    while y != 0 {
        let remainder = x % y;
        x = y;
        y = remainder;
    }
    return x;
}

export func lcm(a: int, b: int) -> int {
    if a == 0 || b == 0 {
        return 0;
    }
    return abs(idiv(a * b, gcd(a, b)));
}

export func is_prime(n: int) -> bool {
    if n < 2 {
        return false;
    }
    if n < 4 {
        return true;
    }
    if n % 2 == 0 {
        return false;
    }

    let i = 3;
    while i * i <= n {
        if n % i == 0 {
            return false;
        }
        i = i + 2;
    }
    return true;
}

export func lerp(a: float, b: float, t: float) -> float {
    return a + (b - a) * t;
}

export func to_degrees(radians: float) -> float {
    return radians * (180.0 / 3.141592653589793);
}

export func to_radians(degrees: float) -> float {
    return degrees * (3.141592653589793 / 180.0);
}

// Exemple d'usage combinant Result et `match`, comme demandé : voir
// aussi std/collections.ks et std/fs.ks pour la même idée appliquée à
// d'autres domaines.
export func describe_sqrt(value: float) -> str {
    match safe_sqrt(value) {
        Ok(root) => {
            return "sqrt(" + str(value) + ") = " + str(root);
        }
        Err(message) => {
            return "erreur: " + message;
        }
    }
}
