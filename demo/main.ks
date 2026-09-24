enum Color {
    Red,
    Green,
    Blue
}

enum Status {
    Pending,
    Running,
    Finished

    func is_finished() -> bool {
        return this == Status.Finished;
    }
}

let color: Color = Color.Red;
let status: Status = Status.Finished;

println(color);
println(status);
println(status.is_finished());
println(Color.Blue == color);
