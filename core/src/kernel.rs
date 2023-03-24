use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use anyhow::{anyhow, Context, Error, Result};
use pulldown_cmark::{CowStr, Event};
use quote::quote;
use syn::{
    parse::{discouraged::Speculative, Parse},
    Expr, Item, Stmt,
};

pub struct File(pub Vec<Code>);

#[derive(Default, Debug)]
pub struct Code {
    items: Vec<Item>,
    top_levels: Vec<TopLevel>,
}

#[derive(Debug)]
pub enum TopLevel {
    Stmt(Stmt),
    Expr(Expr),
}

impl<'a> TryFrom<Vec<Event<'a>>> for Code {
    type Error = Error;

    fn try_from(events: Vec<Event<'a>>) -> Result<Self> {
        let code_ev = events
            .get(1)
            .context("can't find code event in: {events:?}")?;
        match code_ev {
            Event::Text(CowStr::Borrowed(code_string)) => Ok(syn::parse_str(code_string)?),
            _ => Err(anyhow!("can't find code string in: {code_ev:?}")),
        }
    }
}

impl Parse for Code {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut code = Code::default();
        while !input.is_empty() {
            {
                let fork = input.fork();
                if let Ok(item) = fork.parse::<Item>() {
                    code.items.push(item);
                    input.advance_to(&fork);
                    continue;
                }
            }
            {
                let fork = input.fork();
                if let Ok(stmt) = fork.parse::<Stmt>() {
                    code.top_levels.push(TopLevel::Stmt(stmt));
                    input.advance_to(&fork);
                    continue;
                }
            }
            {
                let fork = input.fork();
                if let Ok(expr) = fork.parse::<Expr>() {
                    code.top_levels.push(TopLevel::Expr(expr));
                    input.advance_to(&fork);
                    continue;
                }
            }
        }
        Ok(code)
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
                println!("{:#?}", eval_context())
            }
        };
        writeln!(f, "{main}")?;

        Ok(())
    }
}
