use ppx_impl::parse_string;

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
        ).unwrap().trim(),
        "5 4"
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
        ).unwrap().trim(),
        "hello world"
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
fn test_too_many_parameters() {
    match parse_string("", std::env::current_dir().unwrap(), [""].into_iter()) {
        Err(ppx_impl::Error::UnusedParameters) => {},
        _ => panic!("Expected UnusedParameters error")
    }
}

#[test]
fn test_param_with_paren() {
    let res = parse_string("#define TEST(a) a\nTEST(b(c))", std::env::current_dir().unwrap(), std::iter::empty()).unwrap();
    assert_eq!(res, " b(c)");
}

#[test]
fn test_pragma_once() {
    let res = parse_string(r#"
#include "pragma_once.txt"
#include "pragma_once.txt"
"#,
        std::env::current_dir().unwrap().join("tests"),
        std::iter::empty()
    ).unwrap();
    assert_eq!(res.trim(), "HELLO");
}

#[test]
fn test_paste_left() {
    let res = parse_string("#define TEST(a) b##a\nTEST(c)", std::env::current_dir().unwrap(), std::iter::empty()).unwrap();
    assert_eq!(res.trim(), "bc");
}

#[test]
fn test_paste_right() {
    let res = parse_string("#define TEST(a) a##b\nTEST(c)", std::env::current_dir().unwrap(), std::iter::empty()).unwrap();
    assert_eq!(res.trim(), "cb");
}

#[test]
fn test_paste_middle() {
    let res = parse_string("#define TEST(a) b##a##c\nTEST(d)", std::env::current_dir().unwrap(), std::iter::empty()).unwrap();
    assert_eq!(res.trim(), "bdc");
}

#[test]
fn test_if_true() {
    let res = parse_string(r#"
#if true
OK
#endif
"#,
        std::env::current_dir().unwrap(),
        std::iter::empty()
    ).unwrap();
    assert_eq!(res.trim(), "OK");
}

#[test]
fn test_if_false() {
    let res = parse_string(r#"
#if false
OK
#endif
"#,
        std::env::current_dir().unwrap(),
        std::iter::empty()
    ).unwrap();
    assert_eq!(res.trim(), "");
}

#[test]
fn test_else() {
    let res = parse_string(r#"
#if false
NOK
#else
OK
#endif
"#,
        std::env::current_dir().unwrap(),
        std::iter::empty()
    ).unwrap();
    assert_eq!(res.trim(), "OK");
}

#[test]
fn test_elif() {
    let res = parse_string(r#"
#if false
NOK
#elif true
OK
#endif
"#,
        std::env::current_dir().unwrap(),
        std::iter::empty()
    ).unwrap();
    assert_eq!(res.trim(), "OK");
}

#[test]
fn test_elif2() {
    let res = parse_string(r#"
#if true
OK
#elif false
NOK
#endif
"#,
        std::env::current_dir().unwrap(),
        std::iter::empty()
    ).unwrap();
    assert_eq!(res.trim(), "OK");
}

#[test]
fn test_elif_after_else() {
    let res = parse_string(r#"
#if false
A
#else
B
#elif true
!
#endif
"#,
        std::env::current_dir().unwrap(),
        std::iter::empty()
    );
    match res {
        Err(ppx_impl::Error::ElifAfterElse) => {},
        _ => panic!("No error thrown")
    }
}

#[test]
fn test_if_param() {
    let res = parse_string(r#"
#param PARAM
#if PARAM
OK
#endif
"#,
        std::env::current_dir().unwrap(),
        ["1"].into_iter()
    ).unwrap();
    assert_eq!(res.trim(), "OK");
}

#[cfg(feature = "vfs")]
#[test]
fn test_feature_vfs() {
    use ppx_impl::parse_vfs;

    let fs = vfs::MemoryFS::new();
    let root: vfs::VfsPath = fs.into();

    root.join("main.txt").unwrap()
        .create_file().unwrap()
        .write_all(b"
#param A
A
#include \"template.txt\" World
").unwrap();

    root.join("template.txt").unwrap()
        .create_file().unwrap()
        .write_all(b"
#param B
B
").unwrap();

    let result = parse_vfs(
        root.join("main.txt").unwrap(),
        root,
        ["Hello"].into_iter()
    ).unwrap();

    assert_eq!(
        result,
        r#"
Hello

World
"#
    )
}
