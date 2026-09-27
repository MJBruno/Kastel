let list: List<int> = []

list.add(400)
list.add(33)
list.add(210)
list.add(240)
list.add(72)
list.add(50)
list.add(89)

let number_failed: int = 0

for i in list {
    match i {
        i if i < 30 => {
            number_failed += 1;
            println("{:>04}: [FAILED]",i)
        }
        i if i > 50 => {
            println("{:>04}: [EXCELLENT]",i)
        }
        _ => println("{:>04}: [CHECKING!]",i)
    }
}

println("RESULT: {} failed in {} collections", number_failed, list.size())