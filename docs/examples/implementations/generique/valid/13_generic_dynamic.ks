// Passage explicite par dynamic.
func identity<T>(value: T) -> T {
    return value;
}

let dynamic_value: dynamic = 123;
let result: dynamic = identity(dynamic_value);

println(result);
