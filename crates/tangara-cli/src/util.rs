use std::io::Write;

use console::Term;

pub fn confirm(term: &mut Term) -> bool {
    // read key and echo it
    let Ok(char) = term.read_char() else { return false };
    let _ = writeln!(term, "{char}");
    let _ = term.flush();

    // check user response
    match char {
        'y' | 'Y' => true,
        _ => false,
    }
}
