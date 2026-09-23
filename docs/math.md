# `std.math` — Référence de la bibliothèque mathématique de Kastel

`std.math` fournit les primitives numériques, la théorie des nombres, les statistiques, la géométrie, les vecteurs, les matrices, les probabilités, l'aléatoire, les nombres complexes et les transformées de Fourier usuelles.

Les primitives numériques sensibles à la stabilité numérique sont implémentées nativement en Rust. Les algorithmes de niveau bibliothèque sont implémentés en Kastel.

## Import

```kastel
import std.math;

println(math.sqrt(2));
println(math.gcd(48, 18));
println(math.PI);
```

Pour la classe complexe :

```kastel
import std.math.Complexe;

let z = new Complexe(3, 4);
println(z.magnitude());
```

## Conventions numériques

Les fonctions numériques générales acceptent `int | float` lorsqu'elles peuvent traiter les deux formes.

Les fonctions qui produisent une valeur réelle normalisent leur résultat en `float`.

Les fonctions entières conservent `int` et utilisent les opérations vérifiées du runtime de Kastel. Un dépassement d'entier n'est pas transformé silencieusement en une valeur fausse.

Les angles trigonométriques sont exprimés en **radians**.

## Constantes

| Nom | Signification |
|---|---|
| `PI` | π |
| `TAU` | 2π |
| `E` | base des logarithmes naturels |
| `SQRT2` | √2 |
| `SQRT_E` | √e |
| `LN2` | ln(2) |
| `LN10` | ln(10) |
| `LOG2E` | log₂(e) |
| `LOG10E` | log₁₀(e) |

## Primitives numériques

```text
abs(x)
floor(x)
ceil(x)
round(x)
trunc(x)
fract(x)
sqrt(x)
cbrt(x)
hypot(x, y)
pow(x, y)
exp(x)
log(x)
log2(x)
log10(x)
```

`floor`, `ceil` et `round` renvoient un entier. `trunc` tronque vers zéro.

## Trigonométrie

```text
sin(x)
cos(x)
tan(x)
sec(x)
csc(x)
cot(x)
asin(x)
acos(x)
atan(x)
atan2(y, x)
```

`atan2(y, x)` conserve le quadrant du vecteur `(x, y)` et doit être préféré à `atan(y / x)` pour les directions 2D.

## Hyperboliques

```text
sinh(x)
cosh(x)
tanh(x)
asinh(x)
acosh(x)
atanh(x)
```

## Angles

```text
to_radians(degrees)
to_degrees(radians)
```

## Bornage et interpolation

```text
sign(x)
clamp(value, low, high)
lerp(start, stop, t)
inverse_lerp(start, stop, value)
map_range(value, in_min, in_max, out_min, out_max)
smoothstep(edge0, edge1, value)
smootherstep(edge0, edge1, value)
```

Formule de `lerp` :

```text
a + (b - a) × t
```

## Tests flottants

```text
is_nan(x)
is_infinite(x)
is_finite(x)
approx_eq(a, b, epsilon)
absolute_error(value, exact)
relative_error(value, exact)
```

`approx_eq` doit être utilisée pour comparer des résultats flottants soumis aux erreurs d'arrondi.

## Théorie des nombres

```text
gcd(a, b)
lcm(a, b)
is_prime(n)
prime_factors(n)
factorial(n)
fibonacci(n)
permutations(n, r)
combinations(n, r)
```

## Arithmétique entière orientée informatique

```text
is_power_of_two(n)
next_power_of_two(n)
bit_length(n)
popcount(n)
leading_zeros(n)
trailing_zeros(n)
```

Ces fonctions utilisent la représentation entière 64 bits de Kastel.

## Statistiques

Entrées principales : `List<int | float>`.

```text
sum(items)
product(items)
mean(items)
average(items)
weighted_mean(values, weights)
median(items)
mode(items)
min_of(items)
max_of(items)
variance(items)
variance_sample(items)
std_dev(items)
sample_std_dev(items)
covariance(a, b)
correlation(a, b)
geometric_mean(items)
harmonic_mean(items)
```

`variance` est la variance de population et divise par `n`.

`variance_sample` est la variance d'échantillon et divise par `n - 1`.

Les statistiques qui exigent au moins une observation lèvent une erreur sur une liste vide.

## Résolution d'équations

```text
solve_linear(a, b)
quadratic_discriminant(a, b, c)
solve_quadratic(a, b, c)
```

Pour `ax² + bx + c = 0`, `solve_quadratic` renvoie une liste de racines réelles :

```text
0 racine    -> []
1 racine    -> [x]
2 racines   -> [x1, x2]
```

## Géométrie

```text
distance_2d(x1, y1, x2, y2)
distance_3d(x1, y1, z1, x2, y2, z2)
manhattan_distance_2d(...)
chebyshev_distance_2d(...)
pythagoras(a, b)
rectangle_area(width, height)
rectangle_perimeter(width, height)
triangle_area(base, height)
triangle_area_heron(a, b, c)
circle_area(radius)
circle_circumference(radius)
sphere_volume(radius)
sphere_surface_area(radius)
cylinder_volume(radius, height)
```

## Vecteurs

Les vecteurs sont représentés par des listes numériques :

```kastel
let a = [1, 2, 3];
let b = [4, 5, 6];
```

API :

```text
dot(a, b)
cross(a, b)
magnitude(vector)
normalize(vector)
vector_distance(a, b)
project(a, b)
angle_between(a, b)
rotate_2d(vector, angle)
reflect(vector, normal)
```

## Matrices

Les matrices sont représentées par des listes de lignes :

```kastel
let matrix = [
    [1, 2],
    [3, 4]
];
```

API :

```text
matrix_rows(matrix)
matrix_cols(matrix)
transpose(matrix)
matrix_add(a, b)
matrix_sub(a, b)
matrix_mul(a, b)
matrix_vector_mul(matrix, vector)
trace(matrix)
determinant_2x2(matrix)
determinant(matrix)
inverse_2x2(matrix)
inverse(matrix)
```

`determinant(matrix)` utilise une expansion récursive : elle convient aux petites matrices, mais n'est pas destinée aux grands calculs scientifiques.

`inverse(matrix)` utilise une élimination de Gauss-Jordan avec recherche de pivot non nul.

## Probabilités et distributions

```text
uniform_pdf(x, low, high)
erf(x)
normal_pdf(x, mean, stddev)
normal_cdf(x, mean, stddev)
bernoulli_pmf(success, probability)
binomial_pmf(n, k, probability)
poisson_pmf(k, lambda)
```

## Fonctions orientées machine learning

```text
sigmoid(x)
relu(x)
softmax(items)
entropy(probabilities)
```

`softmax` soustrait la valeur maximale avant l'exponentiation afin de limiter les risques de débordement numérique.

## Aléatoire

Primitives globales :

```text
rand()
rand_int(max)
rand_range(low, high)
```

Fonctions de niveau bibliothèque :

```text
random_range(low, high)
random_bool()
random_sign()
choice(items)
shuffle(items)
random_normal(mean, stddev)
```

`random_range(low, high)` produit une valeur réelle dans `[low, high)`.

`rand_range(low, high)` produit un entier dans `[low, high)`.

`shuffle(items)` travaille sur une copie et ne modifie pas la liste passée en argument.

`random_normal` utilise Box-Muller.

## Nombres complexes

La classe `Complexe` fournit :

```text
initialize(real, imaginaire)
add(other)
subtract(other)
multiply(other)
divide(other)
scale(factor)
conjugate()
magnitude()
phase()
exp()
to_string()
```

Constructeur polaire :

```text
complex_from_polar(radius, angle)
```

## Fourier

```text
dft(values)
fft(values)
fft_real(values)
ifft(values)
```

`dft` réalise la transformée de Fourier discrète directement en `O(n²)`.

`fft` utilise une décomposition Cooley-Tukey récursive et exige une longueur qui soit une puissance de deux.

`fft_real` est un raccourci pour transformer une liste réelle en nombres complexes puis appliquer `fft`.

`ifft` reconstruit la transformée inverse avec conjugaison et normalisation.

## Règles de domaine

Certaines fonctions valident explicitement leur domaine :

```text
sqrt / log / trig inverse
        -> comportement IEEE du runtime pour les flottants

factorial / fibonacci
        -> n >= 0

is_prime / prime_factors
        -> entiers

normal_pdf / normal_cdf
        -> stddev > 0

random_normal
        -> stddev > 0

weighted_mean
        -> tailles identiques et somme des poids non nulle

normalize
        -> vecteur non nul

inverse / determinant
        -> matrice carrée
```

Les opérations entières qui dépassent `i64` doivent conserver la politique générale de Kastel : **erreur explicite de dépassement**, jamais wrapping implicite.

## Architecture d'implémentation

```text
Kastel source
    │
    ├── std/math.ks
    │      ├── API haut niveau
    │      ├── statistiques
    │      ├── géométrie
    │      ├── matrices
    │      ├── probabilités
    │      ├── complexes
    │      └── Fourier
    │
    └── src/stdlib/math.rs
           ├── opérations numériques natives
           ├── trigonométrie
           ├── exponentielles/logarithmes
           ├── classification flottante
           └── générateur aléatoire
```

Cette séparation garde les opérations sensibles et coûteuses au niveau natif, tout en permettant d'étendre l'API en Kastel sans modifier la VM pour chaque nouvelle formule.
