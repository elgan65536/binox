use binox::binox_interpreter::run_interpreter;
use binox::make_files::create_default_files;

const MAKE_FILES: bool = false;

fn main() {
    if MAKE_FILES {
        create_default_files();
    } else {
        run_interpreter();
    }
}
