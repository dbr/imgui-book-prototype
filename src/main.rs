fn main() {
    let meta: Vec<imgui_book_shared::ExampleSnippet> = serde_json::from_slice(rdr::get_metadata()).unwrap();
    dbg!(meta);
    rdr::generate_all();
}
