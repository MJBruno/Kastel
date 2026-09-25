class Parent<T> {}
class Child: Parent<int> {}

let child: Child = new Child();
let invalid: Parent<str> = child;
