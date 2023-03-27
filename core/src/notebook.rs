use std::process;
use std::str;

use anyhow::Result;
use pulldown_cmark::{CowStr, Event, Options, Parser};
use pulldown_cmark_to_cmark::cmark;

#[derive(Debug)]
pub struct Notebook<'a> {
    pub cells: Vec<Cell<'a>>,
}

#[derive(Debug)]
pub enum Cell<'a> {
    Markdown(Vec<Event<'a>>),
    RustCode(Vec<Event<'a>>),
    EvaluatedRustCode(EvaluatedRustCode<'a>),
}

#[derive(Debug)]
pub struct EvaluatedRustCode<'a> {
    pub events: Vec<Event<'a>>,
    pub output: Result<process::Output>,
}

impl<'a> Notebook<'a> {
    pub fn new() -> Self {
        Notebook { cells: vec![] }
    }
}

impl<'a> From<&'a str> for Notebook<'a> {
    fn from(markdown: &'a str) -> Self {
        let parser = Parser::new_ext(markdown, Options::all());
        parser.collect()
    }
}

impl<'a> TryFrom<&Notebook<'a>> for String {
    type Error = anyhow::Error;

    fn try_from(notebook: &Notebook<'a>) -> Result<Self, Self::Error> {
        let mut buf = String::new();

        let events = notebook
            .cells
            .iter()
            .flat_map(|cell| cell.output_markdown_event_iter());
        cmark(events, &mut buf)?;

        Ok(buf)
    }
}

impl<'a> FromIterator<Event<'a>> for Notebook<'a> {
    fn from_iter<T: IntoIterator<Item = Event<'a>>>(events: T) -> Self {
        let mut notebook = Notebook::new();
        let mut state = parse::State::new();
        for ev in events {
            state = state.parse(ev, &mut notebook);
        }
        notebook
    }
}

impl<'a> Cell<'a> {
    pub fn markdown_events(self: &Self) -> &Vec<Event> {
        let events = match self {
            Cell::Markdown(events) => events,
            Cell::RustCode(events) => events,
            Cell::EvaluatedRustCode(code) => &code.events,
        };
        events
    }

    pub fn output_markdown_event_iter(self: &Self) -> Box<dyn Iterator<Item = Event> + '_> {
        match self {
            Cell::EvaluatedRustCode(code) => Box::new(code.events.iter().map(|ev| match ev {
                Event::Text(CowStr::Borrowed(code_string)) => {
                    let output_string = match &code.output {
                        Err(err) => format!("{:#?}", err),
                        Ok(output) => str::from_utf8(&output.stdout).unwrap().to_owned(),
                        // format!(
                        //     "status: {}\nstderr: {}\nstdout: {}\n",
                        //     output.status,
                        //     str::from_utf8(&output.stderr).unwrap(),
                        //     str::from_utf8(&output.stdout).unwrap(),
                        // ),
                    }
                    .split("\n")
                    .into_iter()
                    // .map(|line| format!("  {line}"))
                    .collect::<Vec<_>>()
                    .join("\n");

                    Event::Text(
                        format!(
                            "{}\n\n/* rust-markdown-notebook:result\n\n=>\n\n{}\n\n*/",
                            code_string, output_string
                        )
                        .into(),
                    )
                }
                _ => ev.to_owned(),
            })),
            _ => Box::new(self.markdown_events().iter().cloned()),
        }
    }
}

mod parse {
    use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag};

    use super::{Cell, Notebook};

    pub enum State<'a> {
        Markdown(Vec<Event<'a>>),
        RustCode(Vec<Event<'a>>),
    }

    impl<'a> State<'a> {
        pub fn new() -> Self {
            Self::Markdown(vec![])
        }
        pub fn parse(self: Self, ev: Event<'a>, notebook: &mut Notebook<'a>) -> Self {
            match self {
                Self::Markdown(mut events) => match ev {
                    Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(CowStr::Borrowed(
                        "rust",
                    )))) => {
                        if !events.is_empty() {
                            notebook.cells.push(Cell::Markdown(events));
                        }
                        State::RustCode(vec![ev])
                    }
                    Event::Rule => {
                        if !events.is_empty() {
                            notebook.cells.push(Cell::Markdown(events));
                        }
                        Self::Markdown(vec![ev])
                    }
                    _ => {
                        events.push(ev);
                        Self::Markdown(events)
                    }
                },
                Self::RustCode(mut events) => match ev {
                    Event::End(Tag::CodeBlock(CodeBlockKind::Fenced(CowStr::Borrowed("rust")))) => {
                        events.push(ev);
                        notebook.cells.push(Cell::RustCode(events));
                        State::Markdown(vec![])
                    }
                    _ => {
                        events.push(ev);
                        State::RustCode(events)
                    }
                },
            }
        }
    }
}
