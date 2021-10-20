#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ExampleTags {
    /// Example is not compiled
    pub ignore: bool,

    /// The example snippet is compiled but running is skipped
    pub no_run: bool,

    /// Should example throw an error?
    pub should_panic: bool,

    // Display code in output
    pub hide_code: bool,

    // Display image in output
    pub hide_output: bool,

    // Used supplied name for example
    pub name: Option<String>,
}

pub fn tags_from_string(raw: String) -> Option<ExampleTags> {
    let mut tags = ExampleTags {
        // Default values, updated from tags momentarily
        ignore: false,
        no_run: false,
        should_panic: false,
        hide_code: false,
        hide_output: false,
        name: None,
    };

    let chunks: Vec<&str> = raw.split(",").collect();

    if chunks.len() > 0 && &chunks[0] != &"imgui-example" {
        return None;
    } else {
        for t in chunks.iter().skip(1) {
            match t {
                &"ignore" => tags.ignore = true,
                &"no_run" => tags.no_run = true,
                &"should_panic" => tags.should_panic = true,
                &"hide_code" => tags.hide_code = true,
                &"hide_output" => tags.hide_output = true,
                &"hide" => {tags.hide_output = true; tags.hide_code = true},
                unknown => {
                    if let Some(name) = unknown.strip_prefix("name=") {
                        tags.name = Some(name.to_string());
                    } else {
                        panic!("Unknown tag {}", unknown)
                    }
                },
            }
        }

        Some(tags)
    }
}


#[derive(Clone, Debug, serde::Deserialize)]
pub struct ExampleSnippet {
    // Unique identifier
    pub ident: String,

    // Code snippet to run
    pub code: String,

    // Flags like no_run
    pub tags: ExampleTags,
}

impl ExampleSnippet {
    pub fn output_filename(&self) -> String {
        format!("{}.png", &self.ident)
    }
}

use serde::ser::{Serialize, SerializeStruct, Serializer};

impl Serialize for ExampleSnippet {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = ser.serialize_struct("ExampleSnippet", 3)?;

        let code_cleaned = self.code.lines().map(|line| {
            let line = if let Some(line) = line.strip_prefix("#") {
                line.trim_start()
            } else {
                line
            };
            format!("{}\n", line)
        }).collect::<String>();
        s.serialize_field("code", &code_cleaned)?;

        s.serialize_field("ident", &self.ident)?;
        s.serialize_field("tags", &self.tags)?;
        s.end()
    }
}
