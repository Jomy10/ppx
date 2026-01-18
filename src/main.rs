use template_system::parse_string;

fn main() {
    println!("'{}'", parse_string("
        #define TEST 5

        TESTe TEST
        ",
        std::env::current_dir().unwrap(),
        std::iter::empty()
    ).unwrap());
}
