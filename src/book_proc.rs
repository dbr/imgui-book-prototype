use mdbook::book::{Book, BookItem};
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, PreprocessorContext};
use std::io;
use std::process;

fn main() -> anyhow::Result<()> {
    // cheap way to detect the "supports" arg, accepting anything
    if std::env::args().len() > 1 {
        process::exit(0);
    }

    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    let book_version = semver::Version::parse(&ctx.mdbook_version)?;
    let version_req = semver::VersionReq::parse(mdbook::MDBOOK_VERSION)?;

    if !version_req.matches(&book_version) {
        eprintln!(
            "Warning: This mdbook preprocessor was built against version {} of mdbook, \
             but we're being called from version {}",
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = process(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}

enum ParsingState {
    Start,
    InCodeBlock(imgui_book_shared::ExampleTags, Option<String>),
}

fn process_examples(chap: &mut mdbook::book::Chapter, image_dir: &std::path::PathBuf, rel_image_path: String) -> anyhow::Result<String> {
    use pulldown_cmark::{Event, Tag};

    let meta: Vec<imgui_book_shared::ExampleSnippet> = serde_json::from_slice(rdr::get_metadata()).unwrap();

    let mut state = ParsingState::Start;

    use pulldown_cmark as md;
    let parser = pulldown_cmark::Parser::new(&chap.content);

    let mut new_events = vec![];

    for (p, offset) in parser.into_offset_iter() {
        match state {
            ParsingState::Start => {
                // Look for start of fenced (triple-backtick) code block
                match p {
                    md::Event::Start(md::Tag::CodeBlock(md::CodeBlockKind::Fenced(tag))) => {
                        if let Some(tags) = imgui_book_shared::tags_from_string(tag.to_string()) {
                            // Check if it's start of a valid imgui-example blocks
                            state = ParsingState::InCodeBlock(tags, None);
                            // Don't output code block, we do this later
                        } else {
                            // Leave other code blocks untouched
                            new_events.push(
                                md::Event::Start(md::Tag::CodeBlock(md::CodeBlockKind::Fenced(tag)))
                            )
                        }
                    },
                    e => new_events.push(e)
                }
            },

            ParsingState::InCodeBlock(ref start_tag, ref mut cur_ident) => {
                // Look for end of code block
                match p {
                    md::Event::End(md::Tag::CodeBlock(cb)) => {
                        let filename = chap.path.as_ref().unwrap().to_string_lossy();
                        let cleaned = filename.chars().map(|c| if c.is_alphabetic(){ c } else { '_' }).collect::<String>();

                        let ident = format!("{}_{}_{}", cleaned, offset.start, offset.end);

                        for a in &meta {
                            if a.ident == ident {
                                *cur_ident = Some(ident.clone());
                            }
                        }

                        for m in &meta {
                            if cur_ident.as_ref() == Some(&m.ident) {
                                // Processor communicates via stdout,
                                // so we suppress any prints etc in
                                // examples to avoid weird errors
                                let _print_gag = gag::Gag::stdout().unwrap();
                                rdr::generate(&m.ident, image_dir);
                                if !m.tags.hide_code {
                                    new_events.push(md::Event::Start(md::Tag::CodeBlock(md::CodeBlockKind::Fenced("rust".into()))));
                                    new_events.push(md::Event::Text(format!("{}\n", m.code).into()));
                                    new_events.push(md::Event::End(md::Tag::CodeBlock(md::CodeBlockKind::Fenced("rust".into()))));
                                }

                                // Output image anchor link `[snippet_123_ident]: ../_generated/example.png`
                                let rel_image_path = format!("{}/{}",rel_image_path, m.output_filename());
                                new_events.push(
                                    md::Event::Html(
                                        format!("[{}]: {}", &m.ident, &rel_image_path)
                                            .into()
                                    )
                                );

                                if !m.tags.hide_output {

                                    new_events.push(md::Event::Start(md::Tag::Paragraph));
                                    let img = md::Tag::Image(
                                        md::LinkType::Inline,
                                        rel_image_path.clone().into(),
                                        "".into(),
                                    );
                                    new_events.push(md::Event::Start(img.clone()));
                                    new_events.push(md::Event::End(img.clone()));

                                    new_events.push(md::Event::End(md::Tag::Paragraph));
                                }
                            }
                        }

                        state = ParsingState::Start;
                    },
                    md::Event::Text(t) => {
                    },
                    e => new_events.push(e),
                }
            },
        }
    }

    let mut buf = String::new();
    pulldown_cmark_to_cmark::cmark(new_events.iter(), &mut buf, None)
        .map(|_| buf)
        .map_err(|err| anyhow::anyhow!("Markdown serialization failed: {}", err))
}

fn get_relative_img_url(chapter_path: &std::path::PathBuf) -> String {
    let depth = chapter_path.components().count();
    let path: String = std::iter::repeat("../").take(depth).collect();
    format!("{}_generated", path)
}

fn process(ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
    let doc_root = ctx.root.join(&ctx.config.book.src);
    let image_dir = doc_root.join("_generated");
    std::fs::create_dir_all(&image_dir).unwrap();

    book.for_each_mut(|sect| {
        if let BookItem::Chapter(ref mut chapter) = sect {
            if let Some(chap_path) = chapter.path.clone() {
                chapter.name = format!("{}!", chapter.name);
                chapter.content = process_examples(chapter, &image_dir, get_relative_img_url(&chap_path)).unwrap();
                eprintln!("{}", &chapter.content);
            }
        }
    });
    Ok(book)
}
