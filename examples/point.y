module point;

import "math" as math;


struct Point {
    x: float,
    y: float
}

extend Point {
    pub func new(x: float, y: float): Point {
        return Point {
            x, y
        };
    }

    pub func magnitude(): float {
        return math.sqrt(x*x + y*y);
    }
}