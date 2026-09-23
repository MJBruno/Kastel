class Node {
    func initialize(value) {
        this.value = value;
        this.next = None;
    }
}

func build_cycle(size) {
    let head = new Node(0);
    let current = head;
    let i = 1;
    while i < size {
        let node = new Node(i);
        current.next = node;
        current = node;
        i = i + 1;
    }
    current.next = head;
    return head;
}

let head = build_cycle(1000);

let sum = 0;
let node = head;
let count = 0;
while count < 1000 {
    sum = sum + node.value;
    node = node.next;
    count = count + 1;
}

println(sum);
println(count);
