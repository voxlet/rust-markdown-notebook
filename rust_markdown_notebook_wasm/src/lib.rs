use rust_markdown_notebook::{kernel, Notebook};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn to_notebook(source: &str) -> JsValue {
    let notebook = Notebook::try_from(source).unwrap();
    serde_wasm_bindgen::to_value(&notebook).unwrap()
}

#[derive(Debug, Serialize)]
pub struct Output {
    pub stdout: String,
    pub stderr: String,
}

#[wasm_bindgen]
pub fn to_executable_source(rust_source: &str) -> String {
    kernel::Code::try_from(rust_source).unwrap().to_string()
}

#[cfg(test)]
pub mod tests {
    use wasm_bindgen_test::{console_log, wasm_bindgen_test};

    use crate::{to_executable_source, to_notebook};

    #[wasm_bindgen_test]
    fn can_serialize_to_notebook() {
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

        let notebook = to_notebook(markdown_input);
        console_log!("{notebook:#?}");
    }

    #[wasm_bindgen_test]
    fn can_translate_to_executable_source() {
        let source = r#"
let a = 1;

let b = 2;
let c = 3;

fn some_fn() -> i32 {
    42
}

b + c

43

// comment

a + b + c + some_fn()

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

let t = Thing::default();
inc_x(&t)

let mut t = Thing::default();
for i in 0..10 {
    inc_x_mut(&mut t);
}
t
"#;
        let output = to_executable_source(source);
        console_log!("{output:#?}");
    }
}
