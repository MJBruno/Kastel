// GC - cyclic references
// This test is intentionally isolated because cyclic assignment/property syntax
// must match the final Kastel object model.

class Node {
    func init(name) {
        this.name = name;
        this.next = None;
    }
}

let a = new Node("a");
let b = new Node("b");
a.next = b;
b.next = a;

println(a.name);
println(b.name);

// After the last references to a/b are gone, a future GC cycle should be able
// to reclaim the strongly connected component.
