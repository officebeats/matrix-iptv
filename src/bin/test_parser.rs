fn main() {
    let name = "NBA: Hawks (ATL) x Bulls (CHI) start:2025-12-21 20:00:00";
    let parsed = matrix_iptv_lib::parser::parse_stream(name, None);
    println!("Parsed: {:?}", parsed.sports_event);
}
