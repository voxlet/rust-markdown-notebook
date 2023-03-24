use std::{
    collections::hash_map::DefaultHasher,
    env, fs,
    hash::{Hash, Hasher},
};

use notebook::Notebook;
use pulldown_cmark::{Options, Parser};

mod kernel;
mod notebook;

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
```

```rust
a + b + c + some_fn()
```

---

we're done

"#;

    let parser = Parser::new_ext(markdown_input, Options::all());

    let notebook: Notebook = parser.collect();
    let mut file = kernel::File(Vec::<kernel::Code>::new());
    for cell in notebook.cells {
        match cell {
            notebook::Cell::RustCode(events) => {
                let code = kernel::Code::try_from(events).unwrap();
                file.push(code);
                let source = file.to_string();
                println!("{}", &source);

                let dir = env::temp_dir();
                let mut hasher = DefaultHasher::new();
                source.hash(&mut hasher);
                let filename = hasher.finish();

                let path = dir.join(filename.to_string());
                println!("writing to {path:?}");
                fs::write(path, source).unwrap();
                println!("--------");
            }
            _ => {}
        }
    }
}
