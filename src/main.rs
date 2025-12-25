use dioxus::prelude::*;
use reqwest;
use regex::{Regex, Replacer};
use std::iter::zip;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

const BOOKS: &str = include_str!("books.txt");

#[component]
fn TextInputField(id: &'static str, label: &'static str, sig: Signal<String>) -> Element {
    rsx! {
        div {
            label { "{label}" }
            input {
                id,
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

#[component]
fn App() -> Element {
    let book_options: Vec<_> = BOOKS.lines().map(|s| s.to_string()).collect();

    // TODO remember api key with cookie?
    let api_key = use_signal(|| String::new());

    let mut book: Signal<String> = use_signal(|| DEFAULT_BOOK.into());
    let reference: Signal<String> = use_signal(|| DEFAULT_VERSE.into());

    let mut single_page: Signal<bool> = use_signal(|| false);

    let mut retrieved_text: Signal<Vec<String>> = use_signal(|| vec! { DEFAULT_TEXT.into() });
    let converted_text = use_memo(move || convert_text(&retrieved_text()));

    let mut err_msg: Signal<Option<String>> = use_signal(|| None);

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        div { id: "title", "Bible Memorization Helper" }
        div {
            id: "inputs",
            // TODO validate
            TextInputField { id: "api-key", label: "API Key:", sig: api_key.clone() }
            div {
                label { "Book:" }
                select {
                    id: "book",
                    onchange: move |data| {
                        book.set(data.data.value());
                    },
                    for opt in book_options {
                        option {
                            selected: book() == opt.clone(),
                            value: opt,
                            div { "{opt}" },
                        }
                    }
                }
            }
            // TODO validate
            TextInputField { id: "reference", label: "Chapter and Verse:", sig: reference.clone() }
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
                        let b = book();
                        let r = reference();
                        let gotten = get_verses(format!("{b}+{r}"), api_key()).await;
                        if let Ok(text) = gotten { // TODO: add empty check
                            retrieved_text.set(text);
                            err_msg.set(None);
                        } else {
                            retrieved_text.set(vec!());
                            err_msg.set(Some("An error occurred processing your request".to_string())); // TODO: better messaging
                        }
                    },
                    "Submit",
                }
            }
        }
        div {
            class: "printable",

            "{book} {reference}"
        }
        div {
            class: "printable verses",

            "Original Text:",
            for v in retrieved_text() {
                div { "{v}" }
            }
        }
        if !single_page() {
            div { id: "print-page-break" }
        }
        div {
            class: "printable verses",
            
            "Memorization Text:",
            for v in converted_text() {
                div { "{v}" }
            }
        }
        if let Some(msg) = err_msg() {
            div { "Error: {msg}" }
        }
    }
}

fn convert_text(src: &Vec<String>) -> Vec<String> {
    src.into_iter()
        .map(|s| convert_verse_regex(s))
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

fn convert_verse_regex(str: &String) -> String {
    let word_select = Regex::new(r"([a-zA-Z]+)\s*").expect("invalid regex");
    word_select.replace_all(str, WordShortener).to_string()
}

#[derive(serde::Deserialize)]
struct BibleResponse {
    passages: Vec<String>,
}

async fn get_verses(reference: String, api_key: String) -> Result<Vec<String>> {
    let text = reqwest::Client::new()
        .get(format!("https://api.esv.org/v3/passage/text/"))
        .header("Authorization", format!("Token {api_key}"))
        .query(&[
            ("q", reference),
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

    Ok(zip(numbers, verses)
        .map(|(n, v)| format!("{n} {v}"))
        .collect())
}
