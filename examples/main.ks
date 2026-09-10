interface Printable {
    function print();
}

interface Document : Printable {
    function save();
}

class File : Document {
    function save() {
        println("save");
    }
}