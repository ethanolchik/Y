module factorial;

import "io" as io;

func factorial(n: int) -> int {
    let n1: int = n;
    let n2: int = n;

    // Future:
    // let (n1, n2): (int, int) = (n1, n1);

    while (n1 > 1) {
        n1 -= 1;
        n2 *= n1;
    }

    return n2;
}

func main() {
    // Future:
    // let num: int = io.input("").parse<int>()

    let num: int = io.input("Enter a number: ") as int;

    io.print("\(num)! = \(factorial(num))");
}