use pulldown_cmark::Event;

#[derive(Debug)]
pub struct Notebook<'a> {
    pub cells: Vec<Cell<'a>>,
}

#[derive(Debug)]
pub enum Cell<'a> {
    Markdown(Vec<Event<'a>>),
    RustCode(Vec<Event<'a>>),
}

impl<'a> Notebook<'a> {
    pub fn new() -> Self {
        Notebook { cells: vec![] }
    }
}

impl<'a> FromIterator<Event<'a>> for Notebook<'a> {
    fn from_iter<T: IntoIterator<Item = Event<'a>>>(iter: T) -> Self {
        parse(iter.into_iter())
    }
}

fn parse<'a, I>(events: I) -> Notebook<'a>
where
    I: Iterator<Item = Event<'a>>,
{
    let mut notebook = Notebook::new();
    let mut state = parse::State::new();
    for ev in events {
        state = state.parse(ev, &mut notebook);
    }
    notebook
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
                    Event::End(Tag::CodeBlock(CodeBlockKind::Fenced(CowStr::Borrowed(
                        "rust",
                    )))) => {
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
