// Enum générique : instanciation de plusieurs types.
enum Result<T, E> {
    Ok,
    Error
}

let ok_int: Result<int, str> = Result.Ok;
let ok_float: Result<float, str> = Result.Ok;
let error_int: Result<int, str> = Result.Error;

// Les variables ci-dessus doivent être acceptées par le type-checker.
println(true);
