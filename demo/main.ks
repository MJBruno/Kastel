interface Printable {
    func render() -> str;
}

class Document: Printable {

    func initialize() {
        
    }
    
    func render() -> str {
        return "Hello depuis render document"
    }
}


func render<T: Printable>(value: T) -> str {
    return value.render();
}



let document = new Document()

let r: str = render(document)

println(r)