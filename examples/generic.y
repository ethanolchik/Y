module generic;

struct Foo<T> {
    x: T
}

func main() {
    let test: Foo<Vec<int>>;
}