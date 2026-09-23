import std.math;
import std.math.Complexe;

println(round(math.PI * 1000000) / 1000000);
println(math.gcd(48, 18));
println(math.lcm(12, 18));
println(math.is_prime(97));
println(math.prime_factors(180));
println(math.factorial(5));
println(math.fibonacci(10));
println(math.permutations(5, 2));
println(math.combinations(5, 2));
println(math.is_power_of_two(1024));
println(math.bit_length(1024));
println(math.popcount(31));
println(math.leading_zeros(1024));
println(math.trailing_zeros(40));

println(round(math.to_degrees(math.to_radians(180))));
println(math.log_base(8, 2));
println(round(math.cbrt(-8)));
println(round(math.sec(0)));
println(round(math.csc(math.PI / 2)));
println(round(math.cot(math.PI / 4)));
println(math.clamp(15, 0, 10));
println(math.lerp(0, 10, 0.5));
println(math.smoothstep(0, 1, 0.5));

println(math.mean([1, 2, 3, 4]));
println(math.median([1, 2, 3, 4]));
println(math.variance([1, 2, 3, 4]));
println(round(math.std_dev([1, 2, 3, 4]) * 1000000) / 1000000);
println(math.weighted_mean([1, 2, 3], [1, 1, 2]));
println(math.mode([1, 2, 2, 3]));

println(math.distance_2d(0, 0, 3, 4));
println(math.manhattan_distance_2d(0, 0, 3, 4));
println(math.chebyshev_distance_2d(0, 0, 3, 4));
println(math.dot([1, 2, 3], [4, 5, 6]));
println(math.cross([1, 2, 3], [4, 5, 6]));
println(math.determinant_2x2([[1, 2], [3, 4]]));
println(math.matrix_mul([[1, 2], [3, 4]], [[5, 6], [7, 8]]));

println(round(math.normal_pdf(0, 0, 1) * 1000000) / 1000000);
println(math.normal_cdf(0, 0, 1));
println(math.bernoulli_pmf(true, 0.25));
println(math.binomial_pmf(4, 2, 0.5));
println(math.poisson_pmf(2, 1));
println(math.sigmoid(0));
println(math.relu(-5));
println(round(math.erf(1) * 1000000) / 1000000);

let z = new Complexe(3, 4);
println(z.magnitude());
println(round(z.phase() * 1000000) / 1000000);
println(math.solve_quadratic(1, -3, 2));
