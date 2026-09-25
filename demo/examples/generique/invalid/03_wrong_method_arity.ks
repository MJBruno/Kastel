class Holder {
    func echo<T>(value: T) -> T {
        return value;
    }
}

let holder = new Holder();
let value = holder.echo<int, str>(10);
