class Point {
    func initialize(x, y) {
        this.x = x;
        this.y = y;
    }

    func to_string() {
        return format("({}, {})", this.x, this.y);
    }

    func add(other) {
        return new Point(this.x + other.x, this.y + other.y);
    }
}

let p1 = new Point(1, 2);
let p2 = new Point(3, 4);
let p3 = p1.add(p2);

println(p1.to_string());
println(p2.to_string());
println(p3.to_string());
println(p3.x);
println(p3.y);
