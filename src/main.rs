use dioxus::prelude::*;
use reqwest;
use regex::{Regex, Replacer};
use std::iter::zip;
use std::sync::LazyLock;
use futures::future::try_join_all;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

const BOOKS_TEXT: &str = include_str!("books.txt");
static BOOKS: LazyLock<Vec<String>> = LazyLock::new(|| { BOOKS_TEXT.lines().map(|s| s.to_string() ).collect() });

static WORD_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([a-zA-Z]+)\s*").expect("invalid regex"));

#[component]
fn TextInputField(class: &'static str, label: &'static str, sig: Signal<String>) -> Element {
    rsx! {
        div {
            label { "{label}" }
            input {
                class,
                type: "text",
                value: "{sig}",
                onchange: move |data| {
                    sig.set(data.data.value());
                },
            }
        }
    }
}

const DEFAULT_BOOK: &str = "John";
const DEFAULT_VERSE: &str = "3:16";
const DEFAULT_TEXT: &str = "For God so loved the world, that he gave his only Son, that whoever believes in him should not perish but have eternal life.";

#[derive(Clone, PartialEq, Eq)]
struct ReferenceData {
    book: String,
    reference: String,
}

#[derive(Clone, PartialEq, Eq)]
struct VerseData {
    verses: Vec<String>,
    words: usize,
}

#[derive(Clone, PartialEq, Eq)]
struct ConvertedVerseData {
    converted: Vec<String>,
}

impl ReferenceData {
    fn init() -> ReferenceData {
        ReferenceData {
            book: DEFAULT_BOOK.to_string(),
            reference: DEFAULT_VERSE.to_string(),
        }
    }
    async fn fetch_verses(&self, api_key: String) -> Result<VerseData> {
        let b = self.book.as_str();
        let r = self.reference.as_str();
        let text = reqwest::Client::new()
            .get(format!("https://api.esv.org/v3/passage/text/"))
            .header("Authorization", format!("Token {api_key}"))
            .query(&[
                ("q", format!("{b}+{r}")),
                ("include-passage-references", false.to_string()),
                ("include-footnotes", false.to_string()),
                ("include-headings", false.to_string()),
            ])
            .send()
            .await?
            .json::<BibleResponse>()
            .await?
            .passages
            .join("");

        let splitter = Regex::new(r"\[[0-9]*\]").expect("invalid regex");

        let verses = splitter.split(&text)
            .skip(1);               // Skip first because it is empty (i.e. before the first verse marker)
        let numbers = splitter.find_iter(&text)
            .map(|s| s.as_str());

        let verses = zip(numbers, verses)
            .map(|(n, v)| format!("{n} {v}"))
            .collect();

        Ok(VerseData::init(verses))
    }
}

impl VerseData {
    fn init(verses: Vec<String>) -> Self {
        let words = verses.iter().map(|v| WORD_REGEX.find_iter(v).count()).sum();
        Self { verses, words, }
    }
    fn convert(&self) -> ConvertedVerseData {
        ConvertedVerseData { converted: convert_text(&self.verses) }
    }
}

#[component]
fn Reference(references: Signal<Vec<ReferenceData>>, idx: usize ) -> Element {
    let books: &Vec<String> = &*BOOKS;
    
    rsx! {
        if references.read().len() > 1 {
            div {
                button {
                    class: "remove-reference",
                    onclick: move |_| async move {
                        references.write().remove(idx);
                    },
                    "-"
                }
            }
        }
        div {
            label { "Book:" }
            select {
                class: "book",
                onchange: move |data| {
                    references.write()[idx].book = data.data.value()
                },
                for opt in books {
                    option {
                        selected: references.read()[idx].book == opt.clone(),
                        value: opt.clone(),
                        div { "{opt}" },
                    }
                }
            }
        }
        // TODO validate
        div {
            label { "Chapter and Verse:" }
            input {
                class: "reference",
                type: "text",
                value: "{references.read()[idx].reference}",
                onchange: move |data| {
                    references.write()[idx].reference = data.data.value()
                },
            }
        }
    }
}

#[component]
fn Verses(
    references: Signal<Vec<ReferenceData>>,
    verses: Signal<Vec<VerseData>>,
    idx: usize,
    show_word_count: Signal<bool>
) -> Element {
    rsx! {
        div {
            class: "printable",

            "{references()[idx].book} {references()[idx].reference}"
            if show_word_count() {
                " ({verses()[idx].words} words)"
            }
        }

        div {
            class: "printable verses",

            for v in &verses()[idx].verses {
                div { "{v}" }
            }
        }
    }
}

#[component]
fn ConvertedVerses(
    references: Signal<Vec<ReferenceData>>,
    verses: Signal<Vec<VerseData>>,
    converted: Memo<Vec<ConvertedVerseData>>,
    idx: usize,
    show_word_count: Signal<bool>,
) -> Element {
    rsx! {
        div {
            class: "printable",

            "{references()[idx].book} {references()[idx].reference}"
            if show_word_count() {
                " ({verses()[idx].words} words)"
            }
        }

        div {
            class: "printable verses",
            
            for v in &converted()[idx].converted {
                div { "{v}" }
            }
        }
        
    }
}

#[derive(serde::Deserialize)]
struct BibleResponse {
    passages: Vec<String>,
}

#[component]
fn App() -> Element {
    // TODO remember api key with cookie?
    let api_key = use_signal(|| String::new());

    let mut show_word_count = use_signal(|| false);
    let mut single_page = use_signal(|| false);

    let mut err_msg: Signal<Option<String>> = use_signal(|| None);

    let mut references = use_signal(|| vec!(ReferenceData::init()));
    let mut verses = use_signal(||vec!(VerseData::init(vec!(DEFAULT_TEXT.to_string()))));
    let converted = use_memo(move || verses.read().iter().map(VerseData::convert).collect::<Vec<ConvertedVerseData>>() );

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1.0" }
        div { id: "title", "Bible Memorization Helper" }
        div {
            id: "inputs",
            div {
                id: "control-inputs",
                // TODO validate
                TextInputField { class: "api-key", label: "API Key:", sig: api_key.clone() }

                div {
                    label { "Show word count" }
                    input {
                        type: "checkbox",
                        checked: show_word_count,
                        onchange: move |_| {
                            show_word_count.set(!show_word_count());
                        }
                    }
                }

                div {
                    label { "Single page?" }
                    input {
                        type: "checkbox",
                        checked: single_page,
                        onchange: move |_| {
                            single_page.set(!single_page());
                        }
                    }
                }
                div {
                    button {
                        id: "submit",
                        onclick: move |_| async move {
                            err_msg.set(None);
                            let mapped = try_join_all(
                                references().iter().map(|r| r.fetch_verses(api_key().to_string()))
                            ).await;
                            if let Ok(v) = mapped {
                                verses.set(v);
                            } else {
                                err_msg.set(Some("Error occured fetching verse data".to_string()));
                                verses.set(
                                    vec!(VerseData::init(vec!("".to_string())))
                                    .into_iter()
                                    .cycle()
                                    .take(references().len())
                                    .collect());
                            }
                        },
                        "Submit",
                    }
                }
                div {
                    button {
                        id: "add_verse",
                        onclick: move|_| async move {
                            references.write().push(ReferenceData::init());
                            verses.write().push(VerseData::init(vec!(DEFAULT_TEXT.to_string())))
                        },
                        "+"
                    }
                }
            }

            div {
                for idx in 0..references().len() {
                    div {
                        class: "reference-input",
                        Reference { references, idx }
                    }
                }
            }
        }

        div { "Original Text:" }
        for idx in 0..references().len() {
            Verses { references, verses, idx, show_word_count}
        }

        if !single_page() {
            div { id: "print-page-break" }
        }

        div { "Memorization Text:" }

        for idx in 0..references().len() {
            ConvertedVerses { references, verses, converted, idx, show_word_count }
        }

        if let Some(msg) = err_msg() {
            div { "Error: {msg}" }
        }
    }
}

fn convert_text(src: &Vec<String>) -> Vec<String> {
    src.into_iter()
        .map(|s| WORD_REGEX.replace_all(s, WordShortener).to_string())
        .collect()
}

struct WordShortener;
impl Replacer for WordShortener {
    fn replace_append(&mut self, caps: &regex::Captures<'_>, dst: &mut String) {
        match &caps[0] {
            "ESV" => dst.push_str("ESV"),
            word => dst.push_str(&word[..1].to_ascii_uppercase()),
        }
    }
}
