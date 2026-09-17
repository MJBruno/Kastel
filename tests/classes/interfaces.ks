interface Shape {
    func area();
    func perimeter();
}

class Rectangle : Shape {
    func init(width, height) {
        this.width = width;
        this.height = height;
    }

    func area() {
        return this.width * this.height;
    }

    func perimeter() {
        return 2 * (this.width + this.height);
    }
}

let rect = new Rectangle(4, 5);

println(rect.area());
println(rect.perimeter());
println(rect is Shape);
