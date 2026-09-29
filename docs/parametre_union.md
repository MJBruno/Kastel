```javascript
func keep<T>(value: T | None) -> T | None {
    return value;
}

let number: int | None = keep(42);
```
