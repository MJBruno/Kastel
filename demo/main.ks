async func compute(value: int) -> int {
    sleep(1000);
    return value * 2;
}

let task: Task<int> = compute(21);
let result: int = await task;

println(result)