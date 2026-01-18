use ppx::parse_string;

#[test]
fn test_define() {
    assert_eq!(
        parse_string("
#define TEST 5

TESTe TEST",
            std::env::current_dir().unwrap(),
            std::iter::empty()
        ).unwrap(),
"

TESTe 5"
    );
}

#[test]
fn test_define_fn() {
    assert_eq!(
        parse_string("
 #define TEST(a, b) b a
 TEST(4, 5)
",
            std::env::current_dir().unwrap(),
            std::iter::empty()
        ).unwrap(),
        "
   5 4
"
    )
}

#[test]
fn test_define_fn_multiline() {
    assert_eq!(
        parse_string("
#define TEST(a, b) \
    b \
    a
TEST(world, hello)
",
            std::env::current_dir().unwrap(),
            std::iter::empty()
        ).unwrap(),
        "
  hello world
"
    );
}

#[test]
fn test_param() {
    assert_eq!(
        parse_string("
#param A
A
",
            std::env::current_dir().unwrap(),
            ["hello"].into_iter()
        ).unwrap(),
        "
hello
"
    )
}

#[test]
fn test_include() {
    assert_eq!(
        parse_string(r#"
#include "test.txt"
"#,
            std::env::current_dir().unwrap().join("tests"),
            std::iter::empty()
        ).unwrap(),
        "
Included from test.txt!
"
    );
}

#[test]
fn test_include_with_param() {
    assert_eq!(
        parse_string(r#"
#include "test_with_param.txt" hello,world
"#,
            std::env::current_dir().unwrap().join("tests"),
            std::iter::empty()
        ).unwrap(),
        "

hello world
"
    )
}

#[test]
fn too_many_parameters() {
    match parse_string("", std::env::current_dir().unwrap(), [""].into_iter()) {
        Err(ppx::Error::UnusedParameters) => {},
        _ => panic!("Expected UnusedParameters error")
    }
}
