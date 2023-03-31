use std::{iter, str};

use anyhow::Result;
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Options, Parser, Tag};
use pulldown_cmark_to_cmark::cmark;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct Notebook {
    pub cells: Vec<Cell>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", content = "cell")]
pub enum Cell {
    Markdown(String),
    RustCode(String),
    EvaluatedRustCode(EvaluatedRustCode),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvaluatedRustCode {
    pub source: String,
    pub output: String,
    pub compiler_message: String,
}

impl Notebook {
    pub fn new() -> Self {
        Notebook { cells: vec![] }
    }

    pub fn from_events<'a, T: IntoIterator<Item = Event<'a>>>(events: T) -> Result<Self> {
        let mut notebook = Notebook::new();
        let mut state = parse::State::new();
        for ev in events {
            state = state.parse(ev, &mut notebook)?;
        }
        Ok(notebook)
    }
}

fn to_events(markdown: &str) -> Parser {
    Parser::new_ext(markdown, Options::all())
}

impl TryFrom<&str> for Notebook {
    type Error = anyhow::Error;

    fn try_from(markdown: &str) -> Result<Self> {
        Notebook::from_events(to_events(markdown))
    }
}

impl TryFrom<&Notebook> for String {
    type Error = anyhow::Error;

    fn try_from(notebook: &Notebook) -> Result<Self, Self::Error> {
        let mut buf = String::new();

        let events = notebook
            .cells
            .iter()
            .flat_map(|cell| cell.output_markdown_event_iter());
        cmark(events, &mut buf)?;

        Ok(buf)
    }
}

fn fenced_rust_code_block() -> Tag<'static> {
    Tag::CodeBlock(CodeBlockKind::Fenced(CowStr::Inlined(
        "rust".try_into().unwrap(),
    )))
}

fn to_code_events(source: &str) -> impl Iterator<Item = Event> {
    vec![
        Event::Start(fenced_rust_code_block()),
        Event::Text(CowStr::Borrowed(source)),
        Event::End(fenced_rust_code_block()),
    ]
    .into_iter()
}

impl Cell {
    // pub fn markdown_events(self: &Self) -> &Vec<Event> {
    //     let events = match self {
    //         Cell::Markdown(content) => Parser::new_ext(content, Options::all()),
    //         Cell::RustCode(yaml) => &yaml.events,
    //         Cell::EvaluatedRustCode(code) => &code.events,
    //     };
    //     events
    // }

    pub fn output_markdown_event_iter(self: &Self) -> Box<dyn Iterator<Item = Event> + '_> {
        match self {
            Cell::EvaluatedRustCode(code) => Box::new(to_code_events(&code.source).map(|ev| {
                match ev {
                    Event::Text(CowStr::Borrowed(code_string)) => Event::Text(
                        format!(
                            "{}\n\n/* rust-markdown-notebook:result\n\n=>\n\n{}\n\n*/",
                            code_string, code.output
                        )
                        .into(),
                    ),
                    _ => ev.to_owned(),
                }
            })),
            Cell::RustCode(source) => Box::new(to_code_events(source)),
            Cell::Markdown(content) => Box::new(to_events(content)),
        }
    }
}

mod parse {
    use anyhow::{anyhow, Result};
    use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag};
    use pulldown_cmark_to_cmark::cmark;

    use super::{Cell, Notebook};

    pub enum State<'a> {
        Markdown(Vec<Event<'a>>),
        RustCode(String),
    }

    fn to_markdown<'a>(events: &Vec<Event<'a>>) -> Result<String> {
        let mut buf = String::new();
        cmark(events.iter(), &mut buf)?;
        Ok(buf)
    }

    impl<'a> State<'a> {
        pub fn new() -> Self {
            Self::Markdown(vec![])
        }
        pub fn parse(self: Self, ev: Event<'a>, notebook: &mut Notebook) -> Result<Self> {
            match self {
                Self::Markdown(mut events) => match ev {
                    Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(CowStr::Borrowed(
                        "rust",
                    )))) => {
                        if !events.is_empty() {
                            notebook.cells.push(Cell::Markdown(to_markdown(&events)?));
                        }
                        Ok(Self::RustCode(String::new()))
                    }
                    _ => {
                        events.push(ev);
                        Ok(Self::Markdown(events))
                    }
                },
                Self::RustCode(mut source) => match ev {
                    Event::Text(CowStr::Borrowed(code_string)) => {
                        source.push_str(code_string);
                        Ok(Self::RustCode(source))
                    }
                    Event::End(Tag::CodeBlock(CodeBlockKind::Fenced(CowStr::Borrowed("rust")))) => {
                        notebook.cells.push(Cell::RustCode(source));
                        Ok(Self::Markdown(vec![]))
                    }
                    _ => Err(anyhow!("unknown markdown event in fenced rust code block")),
                },
            }
        }
    }
}
