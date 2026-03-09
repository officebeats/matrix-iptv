fn main() {
    let mock_name = "US | MSNBC HEVC";
    
    // Stream layout extraction natively replicated to simulate Wave Terminal bugs
    let mut spans = vec![];
    let row_num = format!("{:>4} ", 1);
    spans.push(row_num);
    spans.push(" ".to_string()); // Used to be `│ `
    
    let c = format!("{:<30} ", "MSNBC HEVC");
    spans.push(c);
    
    println!("DEBUG: Stream constraint outputs - ");
    for span in spans {
        println!("--> '{}' (len {})", span, span.len());
    }

    // Category Grid constraints 
    println!("\nDEBUG: Category 4-column output simulation - ");
    let max_name_len: usize = 20;
    let count: usize = 1205;
    let count_str = format!("{:04}", count); // Used to be `[{:04}]`

    let pre_pad = "  "; // active 
    let fav_marker = " "; 
    let cat_icon = "🎬 "; // 3 bytes, 2 chars visual? Unicode width handling is likely what breaks Wave
    let name_clean = format!("{}{}{} {}", pre_pad, fav_marker, cat_icon, "MOVIES");

    let padded_name = format!("{:<max_name_len$}", name_clean, max_name_len=max_name_len);
    println!("--> '{}' (len {} chars, {} bytes)", padded_name, padded_name.chars().count(), padded_name.len());
    println!("--> '{}' (len {})", count_str, count_str.len());
    println!("--> GAP: '{}' (len {})", "", "".len()); // Changed padding space to empty constraint
}

