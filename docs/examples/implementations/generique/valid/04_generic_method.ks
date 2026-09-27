// Méthode générique indépendante du type de la classe.
class Holder {
    func echo<T>(value: T) -> T {
        return value;
    }
}

let holder = new Holder();

let a: int = holder.echo(77);
let b: str = holder.echo<str>("kastel");

println(a);
println(b);
