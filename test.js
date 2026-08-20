let x = 0

{
        let x = 50
        while (x < 100) {
                x = x + 1

                if (x % 2 == 1) {
                        continue
                }
                print(x)
        }
}
print(x)