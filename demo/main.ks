<<<<<<< HEAD
// examples/overloads_demo.ks
//
// Surcharge par ARITÉ (nombre d'arguments) : méthodes, constructeurs et
// déclarations d'interface. Deux signatures de même nom ET de même arité
// restent interdites (erreur de compilation « déjà déclarée »).
// Lancer avec : kastel examples/overloads_demo.ks

interface Shape {
    func area() -> float;
    func area(scale: float) -> float;
}

class Point {
    // -- Constructeur surchargé -------------------------------------------
    func init() {
        this.x = 0;
        this.y = 0;
    }

    func init(x: int) {
        this.x = x;
        this.y = 0;
    }

    func init(x: int, y: int) {
        this.x = x;
        this.y = y;
    }

    // -- Méthode surchargée -----------------------------------------------
    func move() -> Point {
        return new Point(this.x + 1, this.y + 1);
    }

    func move(dx: int) -> Point {
        return new Point(this.x + dx, this.y);
    }

    func move(dx: int, dy: int) -> Point {
        return new Point(this.x + dx, this.y + dy);
    }

    func to_tuple() -> Tuple<int, int> {
        return (this.x, this.y);
    }
}

let a = new Point();
let b = new Point(5);
let c = new Point(1, 2);

println(a.to_tuple());                  // (0, 0)
println(b.to_tuple());                  // (5, 0)
println(c.to_tuple());                  // (1, 2)

println(c.move().to_tuple());           // (2, 3)
println(c.move(10).to_tuple());         // (11, 2)
println(c.move(10, 20).to_tuple());     // (11, 22)

// -- Déclaration d'interface surchargée ------------------------------------
class Square: Shape {
    func init(side: float) {
        this.side = side;
    }

    func area() -> float {
        return this.side * this.side;
    }

    func area(scale: float) -> float {
        return this.side * this.side * scale;
    }
}

let s: Shape = new Square(3.0);

println(s.area());                      // 9.0
println(s.area(2.0));                   // 18.0

// Refusés (à décommenter pour voir l'erreur) :
//   new Point(1, 2, 3);                // aucun constructeur à 3 arguments
//   c.move(1, 2, 3);                   // aucune surcharge à 3 arguments
//   s.area(1.0, 2.0);                  // idem, via le type Shape
=======
// examples/std_extras_demo.ks
//
// std.json / std.file / std.path / std.os
// Lancer avec : kastel examples/std_extras_demo.ks

import std.json;
import std.file;
import std.path;
import std.os;

// -- json ---------------------------------------------------------------
let payload = {
    name: "Ada",
    age: 36,
    languages: ["Kastel", "Rust"],
    active: true
};

let text = json.encode(payload);
println(text);

// let decoded = json.decode(text);
// println(decoded.get("name"));
// println(decoded.get("languages"));

// // -- file + json.write_file/read_file ------------------------------------
let out = path.join([os.cwd(), "kastel_demo.json"]);

json.write_file(out, payload);
// println(file.read(out));

// let reloaded = json.read_file(out);
// println(reloaded.get("age"));

// file.delete(out);

// // -- path -----------------------------------------------------------------
// println(path.basename(out));     // kastel_demo.json
// println(path.extension(out));    // json
// println(path.stem(out));         // kastel_demo
// println(path.dirname(out));

// // -- os -------------------------------------------------------------------
// println(os.name());
// println(os.arch());
// println(os.args());
>>>>>>> d9a04583b24fbca1c3c82b2fbe5ab774a46c2e4b
