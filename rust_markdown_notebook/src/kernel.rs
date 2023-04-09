use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use anyhow::Result;
use quote::quote;
use syn::{
    parse::{discouraged::Speculative, Parse, ParseStream},
    Expr, Item, Stmt,
};

#[derive(Default)]
pub struct File(pub Vec<Code>);

impl File {
    pub fn new() -> Self {
        File::default()
    }
}

#[derive(Default, Debug)]
pub struct Code {
    items: Vec<Item>,
    top_levels: Vec<TopLevel>,
}

impl TryFrom<&str> for Code {
    type Error = anyhow::Error;

    fn try_from(source: &str) -> Result<Self> {
        let code: Code = syn::parse_str(source)?;
        Ok(code)
    }
}

#[derive(Debug)]
pub enum TopLevel {
    Stmt(Stmt),
    Expr(Expr),
}

// impl<'a> TryFrom<&Vec<Event<'a>>> for Code {
//     type Error = anyhow::Error;

//     fn try_from(events: &Vec<Event<'a>>) -> Result<Self> {
//         // TODO: make source code extraction more robust - assume it's in the 2nd event for now
//         let code_ev = events
//             .get(1)
//             .context("can't find code event in: {events:?}")?;
//         match code_ev {
//             Event::Text(CowStr::Borrowed(code_string)) => Ok(syn::parse_str(code_string)?),
//             _ => Err(anyhow!("can't find code string in: {code_ev:?}")),
//         }
//     }
// }

fn try_parse<T: Parse>(input: &ParseStream) -> syn::Result<T> {
    let fork = input.fork();
    let result = fork.parse::<T>();
    if let Ok(_) = result {
        input.advance_to(&fork);
    }
    result
}

impl Parse for Code {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut code = Code::default();
        while !input.is_empty() {
            // println!("try Item");
            match try_parse::<Item>(&input) {
                Ok(parsed) => {
                    println!("parsed as Item");
                    code.items.push(parsed);
                    continue;
                }
                Err(_err) => {
                    // println!("{err:?}");
                }
            }

            // println!("try Stmt");
            match try_parse::<Stmt>(&input) {
                Ok(parsed) => {
                    println!("parsed as stmt");
                    code.top_levels.push(TopLevel::Stmt(parsed));
                    continue;
                }
                Err(_err) => {
                    // println!("{err:?}");
                }
            }

            // println!("try Expr");
            match try_parse::<Expr>(&input) {
                Ok(parsed) => {
                    println!("parsed as expr");
                    code.top_levels.push(TopLevel::Expr(parsed));
                    continue;
                }
                Err(_err) => {
                    // println!("{err:?}");
                }
            }

            // TODO
            println!("oh no can't parse");
            println!("{input:?}");
            panic!();
        }
        Ok(code)
    }
}

impl Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for item in self.items.iter() {
            writeln!(f, "{}", quote!(#item))?;
        }
        if self.top_levels.is_empty() {
            return Ok(());
        }

        let (last, but_last) = self.top_levels.split_last().unwrap();

        let mut top_levels = but_last
            .iter()
            .map(|t| match t {
                TopLevel::Stmt(stmt) => quote!(#stmt),
                TopLevel::Expr(expr) => quote!(#expr;),
            })
            .collect::<Vec<_>>();

        top_levels.push(match last {
            TopLevel::Stmt(stmt) => quote!(#stmt),
            TopLevel::Expr(expr) => quote!(#expr),
        });

        let eval_context = quote! {
            fn eval_context() -> impl std::any::Any + std::fmt::Debug {
                #(#top_levels)*
            }
        };
        writeln!(f, "{eval_context}")?;

        let main = quote! {
            fn main() {
                print!("{:#?}", eval_context())
            }
        };
        writeln!(f, "{main}")?;

        Ok(())
    }
}

impl Deref for File {
    type Target = Vec<Code>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for File {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for item in self.iter().flat_map(|code| &code.items) {
            writeln!(f, "{}", quote!(#item))?;
        }
        let code_top_levels: Vec<_> = self.iter().flat_map(|code| &code.top_levels).collect();
        if code_top_levels.is_empty() {
            return Ok(());
        }

        let (last, but_last) = code_top_levels.split_last().unwrap();

        let mut top_levels = but_last
            .iter()
            .map(|t| match t {
                TopLevel::Stmt(stmt) => quote!(#stmt),
                TopLevel::Expr(expr) => quote!(#expr;),
            })
            .collect::<Vec<_>>();

        top_levels.push(match *last {
            TopLevel::Stmt(stmt) => quote!(#stmt),
            TopLevel::Expr(expr) => quote!(#expr),
        });

        let eval_context = quote! {
            fn eval_context() -> impl std::any::Any + std::fmt::Debug {
                #(#top_levels)*
            }
        };
        writeln!(f, "{eval_context}")?;

        let main = quote! {
            fn main() {
                print!("{:#?}", eval_context())
            }
        };
        writeln!(f, "{main}")?;

        Ok(())
    }
}

pub mod eval {
    use std::{
        collections::hash_map::DefaultHasher,
        env, fs,
        hash::{Hash, Hasher},
        path::Path,
        process::{self, Command},
    };

    use anyhow::{anyhow, Context, Result};

    use crate::notebook::{Cell, EvaluatedRustCode, Notebook};

    use super::{Code, File};

    fn with_scratch_dir<T>(file: &File, f: impl Fn(&Path, &str) -> Result<T>) -> Result<T> {
        let source = file.to_string();

        let mut hasher = DefaultHasher::new();
        source.hash(&mut hasher);
        let dir = env::temp_dir()
            .join("rust-markdown-notebook-scratch")
            .join(hasher.finish().to_string());

        fs::create_dir_all(&dir).context(format!("create: {dir:?}"))?;
        let res = f(&dir, &source);
        fs::remove_dir_all(&dir).context(format!("remove: {dir:?}"))?;

        res
    }

    pub fn eval_file(file: &File) -> Result<process::Output> {
        with_scratch_dir(&file, |dir, source| {
            if file.len() == 0 {
                return Err(anyhow!("empty file"));
            }

            println!("--------");
            println!("scratch dir: {dir:?}");
            println!("{}", &source);

            // TODO: use user provided Cargo.toml - generate one for now
            fs::write(
                dir.join("Cargo.toml"),
                "[package]\nname = \"tmp\"\nversion = \"0.1.0\"",
            )?;
            let src_dir = dir.join("src");
            fs::create_dir(&src_dir).or_else(|_| anyhow::Ok(()))?;
            fs::write(src_dir.join("main.rs"), source).context("can't write")?;

            let output = Command::new("cargo")
                .arg("run")
                .current_dir(&dir)
                .output()?;

            println!("stderr {:#?}", std::str::from_utf8(&output.stderr));
            println!("");
            println!("=> {:#?}", std::str::from_utf8(&output.stdout)?);
            println!("--------");

            Ok(output)
        })
    }

    pub fn u8s_to_string(buf: &Vec<u8>) -> String {
        std::str::from_utf8(buf).unwrap().to_owned()
    }

    pub fn eval_all_cells(notebook: &mut Notebook) -> Result<()> {
        let mut file = File::new();
        for cell in notebook.cells.iter_mut() {
            match cell {
                Cell::RustCode(source) => {
                    let code = Code::try_from(source.as_ref())?;
                    file.push(code);
                    let (output, compiler_message) = match eval_file(&file) {
                        Ok(out) => (u8s_to_string(&out.stdout), u8s_to_string(&out.stderr)),
                        Err(err) => (String::from("error"), err.to_string()),
                    };
                    *cell = Cell::EvaluatedRustCode(EvaluatedRustCode {
                        source: source.to_owned(),
                        output,
                        compiler_message,
                    });
                }
                _ => {}
            }
        }
        Ok(())
    }
}
