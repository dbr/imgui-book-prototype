use std::io::Write;
use pulldown_cmark as md;
use imgui_book_shared::{ExampleSnippet, ExampleTags};

enum ParsingState {
    Start,
    InCodeBlock(ExampleTags, Option<String>),
}

struct Template<'a> {
    example: &'a ExampleSnippet,
}

impl Template<'_> {
    fn invoke_snippet(&self, mut out: impl std::io::Write) -> Result<(), std::io::Error> {
        if self.example.tags.ignore {
            return Ok(())
        }

        writeln!(&mut out, "imgui_example_{ident}(directory);", ident=self.example.ident)
    }
    fn to_snippet(&self, out: impl std::io::Write) -> Result<(), std::io::Error> {
        if self.example.tags.ignore {
            return Ok(())
        }

        let hb = {
            let mut hb = handlebars::Handlebars::new();

            // Don't do any escaping, not web output
            hb.register_escape_fn(|s| s.into());

            // Error on undefined variables etc
            hb.set_strict_mode(true);
            hb
        };

        let template = r#"
fn imgui_example_{{ident}}(directory: &std::path::PathBuf) {
    let width = 500;
    let height = 500;

    let mut imgui_ctx = imgui::Context::create();

    imgui_ctx.set_ini_filename(None);

    // Cursor
    imgui_ctx.io_mut().mouse_draw_cursor = true;
    imgui_ctx.io_mut().mouse_pos = [200.0, 50.0];

    // Register the default font
    imgui_ctx.fonts().add_font(&[imgui::FontSource::DefaultFontData {
        config: Some(imgui::FontConfig {
            size_pixels: 13.0,
            ..imgui::FontConfig::default()
        }),
    }]);

    // Generate font atlas texture
    // FIXME: Belongs as helper in lib
    let font_pixmap = {
        let mut font_atlas = imgui_ctx.fonts();
        let font_atlas_tex = font_atlas.build_rgba32_texture();

        let mut font_pixmap = tiny_skia::Pixmap::new(font_atlas_tex.width, font_atlas_tex.height).unwrap();

        {
            let data = font_pixmap.pixels_mut();
            for (i, src) in font_atlas_tex.data.chunks(4).enumerate() {
                data[i] =
                    tiny_skia::ColorU8::from_rgba(src[0], src[1], src[2], src[3]).premultiply();
            }
        }

        font_pixmap
    };

    // Set display size
    imgui_ctx.io_mut().display_size = [width as f32, height as f32];
    imgui_ctx.io_mut().display_framebuffer_scale = [1.0, 1.0];


    for frame in 0..2 {
        println!("Frame {}", frame);
        imgui_ctx
            .io_mut()
            .update_delta_time(std::time::Duration::from_millis(20));

        let draw_data: &imgui::DrawData = {
            // New frame
            let ui = imgui_ctx.frame();

            {
                // Start example snippet
{{code}}
                // End example snippet
            };

            ui.render()
        };

        let mut px = tiny_skia::Pixmap::new(width, height).unwrap();
        px.fill(tiny_skia::Color::from_rgba8(89, 89, 89, 255));

        let r = imgui_software_renderer::Renderer::new();
        r.render(&mut px, draw_data, font_pixmap.as_ref());

        // Save output
        let fname = directory.join(&format!("{{ident}}.png"));
        dbg!(&fname);
        px.save_png(fname).unwrap();
    }
}
"#;


        hb.render_template_to_write(template, &self.example, out).unwrap();

        Ok(())
    }
}


fn main(){
    // Get location of book
    let here = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let bookdir = here.join("docs").join("src");

    // Rerun if summary doc changes
    println!("cargo:rerun-if-changed={}", &bookdir.join("SUMMARY.md").to_string_lossy());

    // Parse book and iterate over each chapter
    let book = mdbook::book::load_book(&bookdir, &Default::default()).unwrap();

    // Create generated file
    let path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("imgui_examples.rs");
    dbg!(&path);
    let out_file = std::fs::File::create(&path).unwrap();


    let mut codeblocks = vec![];

    for sec in book.iter() {
        if let mdbook::book::BookItem::Chapter(chap) = sec {
            // Rerun if the source .md files change
            if let Some(p) = &chap.path {
                println!("cargo:rerun-if-changed={}", &bookdir.join(p).to_string_lossy());
            }

            // Parse contents of chapter
            let parser = md::Parser::new(&chap.content);

            // Track state for event-based parser
            let mut state = ParsingState::Start;

            for (p, offset) in parser.into_offset_iter() {
                match state {
                    ParsingState::Start => {
                        // Look for start of fenced (triple-backtick) code block
                        match p {
                            md::Event::Start(md::Tag::CodeBlock(md::CodeBlockKind::Fenced(tag))) => {
                                if let Some(tags) = imgui_book_shared::tags_from_string(tag.to_string()) {
                                    // Check if it's start of a valid imgui-example blocks
                                    state = ParsingState::InCodeBlock(tags, None);
                                } else {
                                    // Ignore other code blocks
                                }
                            },
                            _ => {}
                        }
                    },

                    ParsingState::InCodeBlock(ref start_tag, ref mut contents) => {
                        // Look for end of code block
                        match p {
                            md::Event::End(md::Tag::CodeBlock(_)) => {
                                let filename = chap.path.as_ref().unwrap().to_string_lossy();
                                let cleaned = filename.chars().map(|c| if c.is_alphabetic(){ c } else { '_' }).collect::<String>();

                                codeblocks.push(ExampleSnippet {
                                    ident: format!("{}_{}_{}", cleaned, offset.start, offset.end),
                                    code: contents.clone().unwrap(),
                                    tags: start_tag.clone(),
                                });
                                state = ParsingState::Start;
                            },
                            md::Event::Text(t) => {
                                // Presumably there can be only one text block inside the code block?
                                assert!(contents.is_none(), "found more than one Text event inside code block");
                                *contents = Some(t.to_string());
                            },
                            _ => {},
                        }
                    },
                }
            }
        }
    }

    // Generate code
    writeln!(&out_file, "#[allow(unreachable_code)]").unwrap();
    for cb in &codeblocks {
        Template{example: cb}.to_snippet(&out_file).unwrap();
    }

    writeln!(&out_file, "\n\npub fn generate_all(directory: &std::path::PathBuf) {{").unwrap();
    for cb in &codeblocks {
        Template{example: cb}.invoke_snippet(&out_file).unwrap();
    }
    writeln!(&out_file, "}}").unwrap();

    let indent = "    ";
    writeln!(&out_file, "\n\npub fn generate(name: &str, directory: &std::path::PathBuf) {{").unwrap();
    writeln!(&out_file, "{indent}match name {{", indent=indent).unwrap();
    for cb in &codeblocks {
        writeln!(&out_file, r#"{indent}{indent}"{name}" => {{ "#, name=cb.ident, indent=indent).unwrap();
        write!(&out_file, "{indent}{indent}{indent}", indent=indent).unwrap();
        Template{example: cb}.invoke_snippet(&out_file).unwrap();
        writeln!(&out_file, "{indent}{indent}}},", indent=indent).unwrap();
    }
    writeln!(&out_file, "    _ => panic!(),").unwrap();
    writeln!(&out_file, "    }}").unwrap();
    writeln!(&out_file, "}}").unwrap();

    // Store code block metadata to temp file..
    {
        let meta_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("imgui_examples_meta.json");
        let f = std::fs::File::create(&meta_path).unwrap();
        serde_json::to_writer_pretty(f, &codeblocks).unwrap();

        // ..and include that into the binary for convinience (instead of re-finding the file later)
        writeln!(
            &out_file,
            r#"pub fn get_metadata() -> &'static [u8] {{"#
        ).unwrap();
        writeln!(
            &out_file,
            r#"include_bytes!("{}")"#,
            meta_path.to_str().unwrap()
        ).unwrap();
        writeln!(&out_file, r#"}}"#).unwrap();
    }
}
