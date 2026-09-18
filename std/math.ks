// std/math.ks
//
// Bibliothèque mathématique de Kastel.
//
// Déjà disponibles PARTOUT sans import (fonctions natives globales,
// voir src/stdlib/math.rs) : abs, floor, ceil, round, sqrt, pow, min,
// max, sin, cos, tan, asin, acos, atan, atan2, log, log10, exp, rand,
// rand_int, rand_range. Ce module ne les redéfinit pas — il ajoute
// tout ce que ces primitives ne couvrent pas : constantes, angles,
// hyperboliques, arrondi, interpolation, théorie des nombres,
// combinatoire, statistiques et aléatoire de plus haut niveau.
//
// Usage : from std.math import PI, clamp, gcd, std_dev;

// ------------------------------------------------------------------
// Constantes
// ------------------------------------------------------------------

export const PI = 3.141592653589793;
export const E = 2.718281828459045;
export const TAU = 6.283185307179586; // 2 * PI

// ------------------------------------------------------------------
// Ré-export des natives globales
//
// abs/sqrt/pow/sin/... sont des fonctions natives GLOBALES (voir
// src/stdlib/math.rs) : elles sont déjà appelables partout sans
// import, comme `sqrt(2.0)`. Ces lignes les ré-exportent aussi SOUS
// LE MÊME NOM depuis ce module, pour permettre l'accès qualifié
// `math.sqrt(2.0)` après `import std.math;` — les deux styles
// cohabitent, au choix de l'appelant.
//
// Pourquoi `const x = x;` et pas `func sqrt(x) { return sqrt(x); }` :
// une fonction top-level de ce module serait "pré-déclarée" (hissée)
// avant la compilation de son propre corps — donc `sqrt` à
// l'intérieur de `func sqrt(...)` référencerait la fonction
// elle-même, pas la native (récursion infinie). Un `const`, lui,
// n'est pas hissé : `sqrt` à droite du `=` désigne encore la native
// au moment où cette ligne s'exécute.
// ------------------------------------------------------------------

export const abs = abs;
export const floor = floor;
export const ceil = ceil;
export const round = round;
export const sqrt = sqrt;
export const pow = pow;
export const min = min;
export const max = max;
export const sin = sin;
export const cos = cos;
export const tan = tan;
export const asin = asin;
export const acos = acos;
export const atan = atan;
export const atan2 = atan2;
export const log = log;
export const log10 = log10;
export const exp = exp;
export const rand = rand;

//rand_int(x) renvoie un entier aléatoire dans [0, x), donc rand_int(5) peut renvoyer 0, 1, 2, 3 ou 4.
export const rand_int = rand_int;

//rand_range(low, high) renvoie un flottant aléatoire dans [low, high).
export const rand_range = rand_range;

// ------------------------------------------------------------------
// Angles
// ------------------------------------------------------------------

export func to_radians(degrees) {
    return degrees * PI / 180;
}

export func to_degrees(radians) {
    return radians * 180 / PI;
}

// ------------------------------------------------------------------
// Fonctions hyperboliques
//
// Pas de native dédiée : dérivées de exp(), qui est déjà exacte et
// disponible globalement.
// ------------------------------------------------------------------

export func sinh(x) {
    return (exp(x) - exp(-x)) / 2;
}

export func cosh(x) {
    return (exp(x) + exp(-x)) / 2;
}

export func tanh(x) {
    return sinh(x) / cosh(x);
}

// ------------------------------------------------------------------
// Racines et distances
// ------------------------------------------------------------------

// Racine cubique, y compris pour les nombres négatifs (cbrt(-8) = -2)
// — pow(x, 1/3) seul échoue sur un x négatif (exposant fractionnaire).
export func cbrt(x) {
    if x < 0 {
        return -pow(-x, 1 / 3);
    }

    return pow(x, 1 / 3);
}

export func hypot(x, y) {
    return sqrt(x * x + y * y);
}

// ------------------------------------------------------------------
// Arrondi et signe
// ------------------------------------------------------------------

export func sign(x) {
    if x > 0 {
        return 1;
    }

    if x < 0 {
        return -1;
    }

    return 0;
}

// Contrairement à floor(), tronque vers zéro : trunc(-1.5) = -1,
// alors que floor(-1.5) = -2.
export func trunc(x) {
    if x < 0 {
        return ceil(x);
    }

    return floor(x);
}

export func clamp(value, low, high) {
    if value < low {
        return low;
    }

    if value > high {
        return high;
    }

    return value;
}

// ------------------------------------------------------------------
// Interpolation
// ------------------------------------------------------------------

export func lerp(start, stop, t) {
    return start + (stop - start) * t;
}

// Opération inverse de lerp() : pour quel `t` obtient-on `value`
// entre `start` et `stop` ?
export func inverse_lerp(start, stop, value) {
    return (value - start) / (stop - start);
}

// Reprojette `value` de l'intervalle [in_min, in_max] vers
// [out_min, out_max].
export func map_range(value, in_min, in_max, out_min, out_max) {
    let t = inverse_lerp(in_min, in_max, value);

    return lerp(out_min, out_max, t);
}

// ------------------------------------------------------------------
// Théorie des nombres
// ------------------------------------------------------------------

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

export func is_prime(n) {
    if n < 2 {
        return false;
    }

    if n == 2 {
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

export func factorial(n) {
    if n < 0 {
        throw "factorial: n doit être positif ou nul";
    }

    let result = 1;
    let i = 2;

    while i <= n {
        result = result * i;
        i = i + 1;
    }

    return result;
}

// n-ième terme de Fibonacci, 0-indexé (fibonacci(0) = 0,
// fibonacci(1) = 1).
export func fibonacci(n) {
    if n < 0 {
        throw "fibonacci: n doit être positif ou nul";
    }

    let a = 0;
    let b = 1;
    let i = 0;

    while i < n {
        let next = a + b;
        a = b;
        b = next;
        i = i + 1;
    }

    return a;
}

// ------------------------------------------------------------------
// Combinatoire
//
// Note : entiers 64 bits, pas de grands nombres arbitraires — au-delà
// d'environ n=20, factorial()/permutations() peuvent déborder,
// exactement comme dans la plupart des langages sans bibliothèque de
// bignum dédiée.
// ------------------------------------------------------------------

// Arrangements de r éléments parmi n, ordre compté : n! / (n - r)!
export func permutations(n, r) {
    if r < 0 || r > n {
        return 0;
    }

    let result = 1;
    let i = 0;

    while i < r {
        result = result * (n - i);
        i = i + 1;
    }

    return result;
}

// Combinaisons de r éléments parmi n, ordre non compté :
// n! / (r! * (n - r)!).
export func combinations(n, r) {
    if r < 0 || r > n {
        return 0;
    }

    if r > n - r {
        r = n - r;
    }

    let result = 1;
    let i = 0;

    while i < r {
        result = result * (n - i);
        result = floor(result / (i + 1));
        i = i + 1;
    }

    return result;
}

// ------------------------------------------------------------------
// Agrégats sur des tableaux de nombres
// ------------------------------------------------------------------

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

export func median(items) {
    if items.length == 0 {
        throw "median: le tableau ne doit pas être vide";
    }

    let sorted = items.copy();
    sorted.sort();

    let length = sorted.length;
    let middle = floor(length / 2);

    if length % 2 == 0 {
        return (sorted.get(middle - 1) + sorted.get(middle)) / 2;
    }

    return sorted.get(middle);
}

// Variance de population (divise par n, pas n - 1).
export func variance(items) {
    let n = items.length;

    if n == 0 {
        return 0;
    }

    let mean = average(items);
    let total = 0;

    for item in items {
        let diff = item - mean;
        total = total + diff * diff;
    }

    return total / n;
}

export func std_dev(items) {
    return sqrt(variance(items));
}

// Valeur la plus fréquente (la première rencontrée en cas d'égalité).
export func mode(items) {
    if items.length == 0 {
        throw "mode: le tableau ne doit pas être vide";
    }

    let values = [];
    let counts = [];

    for item in items {
        let index = values.index_of(item);

        if index < 0 {
            values.push(item);
            counts.push(1);
        } else {
            counts.set(index, counts.get(index) + 1);
        }
    }

    let best_index = 0;
    let i = 1;

    while i < counts.length {
        if counts.get(i) > counts.get(best_index) {
            best_index = i;
        }

        i = i + 1;
    }

    return values.get(best_index);
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

// ------------------------------------------------------------------
// Aléatoire de plus haut niveau
//
// Construit sur rand()/rand_int() (natifs).
// ------------------------------------------------------------------

// Flottant uniforme dans [low, high).
export func random_range(low, high) {
    return low + rand() * (high - low);
}

export func choice(items) {
    if items.length == 0 {
        throw "choice: le tableau ne doit pas être vide";
    }

    let index = rand_int(items.length);

    return items.get(index);
}

// Mélange de Fisher-Yates. Ne modifie pas `items` : renvoie une copie
// mélangée.
export func shuffle(items) {
    let result = items.copy();
    let i = result.length - 1;

    while i > 0 {
        let j = rand_int(i);

        let temp = result.get(i);
        result.set(i, result.get(j));
        result.set(j, temp);

        i = i - 1;
    }

    return result;
}

// ------------------------------------------------------------------
// Nombres complexes
//
// Démontre l'autre moitié de la convention d'import : une CLASSE
// vit dans un module comme std.math au même titre qu'une fonction,
// mais s'importe SANS qualifier par le nom du module :
//
//     import std.math.Complexe;
//     let z = new Complexe(3, 4);   // pas new math.Complexe(3, 4)
// ------------------------------------------------------------------

export class Complexe {
    func init(real, imaginaire) {
        this.real = real;
        this.imaginaire = imaginaire;
    }

    func add(other) {
        return new Complexe(this.real + other.real, this.imaginaire + other.imaginaire);
    }

    func subtract(other) {
        return new Complexe(this.real - other.real, this.imaginaire - other.imaginaire);
    }

    func multiply(other) {
        let real = this.real * other.real - this.imaginaire * other.imaginaire;
        let imaginaire = this.real * other.imaginaire + this.imaginaire * other.real;

        return new Complexe(real, imaginaire);
    }

    func conjugate() {
        return new Complexe(this.real, - this.imaginaire);
    }

    func magnitude() {
        return hypot(this.real, this.imaginaire);
    }

    func to_string() {
        if this.imaginaire < 0 {
            return format("{}{}i", this.real, this.imaginaire);
        }

        return format("{} + {}i", this.real, this.imaginaire);
    }
}

