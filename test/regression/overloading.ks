// À placer dans test/regression/overloading.ks
// Couvre : surcharge de fonctions libres (au niveau module) et de
// fonctions locales, dispatch par arité/type.

func greet() -> str {
    return "hello";
}

func greet(name: str) -> str {
    return "hello, " + name;
}

print(greet());          // hello
print(greet("bruno"));   // hello, bruno

func outer() -> int {
    func add(a: int, b: int) -> int { return a + b; }
    func add(a: int, b: int, c: int) -> int { return a + b + c; }

    return add(1, 2) + add(1, 2, 3);
}

print(outer());          // 3 + 6 = 9

class Calc {
    func combine(x: int, y: int) -> int { return x + y; }
    func combine(x: str, y: str) -> str { return x + y; }
}

let c = Calc();
print(c.combine(1, 2));       // 3
print(c.combine("a", "b"));   // ab
