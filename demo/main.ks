interface Shape {
    func area()-> int;
    func perimeter()-> int;
}

class Rectangle : Shape {

    private let width: int = 0;
    private let height: int = 0;

    public func initialize(width:int, height:int) {
        this.width = width;
        this.height = height;
    }

    public func area()->int {
        return this.width * this.height;
    }

    public func perimeter()->int {
        return 2 * (this.width + this.height);
    }
}

let rect:Rectangle = new Rectangle(4, 5);

println(rect.area());
println(rect.perimeter());
println(rect is Shape);