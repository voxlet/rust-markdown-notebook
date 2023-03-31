use rust_markdown_notebook::{eval, Notebook};

fn main() {
    let markdown_input = r#"
# Title

Hello world, this is a ~~complicated~~ *very simple* example.
next line

--

a markdown cell

---

another cell

```rust
let a = 1;
```

## Do Stuff

```rust
let b = 2;
let c = 3;

fn some_fn() -> i32 {
    42
}
```

```rust
b + c
```

## More Stuff

```rust
43

// comment
```

```rust
a + b + c + some_fn()
```

## Complicated Stuff

```rust
#[derive(Default, Debug)]
struct Thing {
    x: i32,
    y: i32,
}

fn inc_x_mut(mut thing: &mut Thing) {
    thing.x = thing.x + 1;
}

fn inc_x(thing: &Thing) -> Thing {
    Thing {
        x: thing.x + 1,
        y: thing.y,
    }
}
```


```rust
let t = Thing::default();
inc_x(&t)
```

```rust
let mut t = Thing::default();
for i in 0..10 {
    inc_x_mut(&mut t);
}
t
```

---

we're done

"#;

    let mut notebook = Notebook::try_from(markdown_input).unwrap();
    eval::eval_all_cells(&mut notebook).unwrap();
    let output = String::try_from(&notebook).unwrap();
    println!("{}", &output);
}
