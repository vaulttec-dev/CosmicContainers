use linux_pop_extension::greeting;
use std::io::{self, Write};

fn main() {
    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{}", greeting()).expect("stdout");
}
