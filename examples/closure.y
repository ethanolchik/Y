module closure;

import "io" as io;

func main() {
    let add: (int, int) -> int = |x: int, y: int| int { return x + y; };

    io.println("1 + 2 = \(add(1, 2))");
}