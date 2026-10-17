use sailc::compile;

fn program(body:&str)->String {format!("module| Test\nfun| > main > int\n{body}\n|fun\n|module\n")}
#[test]
fn supplied_examples_compile() {
    for (name,source) in [
        ("hello",include_str!("../examples/hello.sl")),
        ("euclidean",include_str!("../examples/euclidean.sl")),
        ("primes",include_str!("../examples/primes.sl")),
        ("classes",include_str!("../examples/classes.sl")),
        ("generics",include_str!("../examples/generics.sl")),
        ("vectors",include_str!("../examples/vectors.sl")),
        ("files",include_str!("../examples/files.sl")),
    ] {assert!(compile(source).is_ok(),"{name}: {:?}",compile(source).err());}
}
#[test]
fn assignment_is_left_to_right() {
    let ir=compile(&program("int a(1)\na + 2 > a\na > out")).unwrap();
    assert!(ir.contains("add i64"));assert!(ir.contains("call void @sail_print_int"));
}
#[test]
fn greater_than_is_comparison_in_when() {
    let ir=compile(&program("int a(4) b(2)\nwhen| a > b\n'yes' > out\n|when")).unwrap();
    assert!(ir.contains("icmp sgt"));
}
#[test]
fn nested_blocks_are_not_early_terminated() {
    compile(&program("repeat| false\nrepeat| false\nwhen| true\nwhen| false\nbreak\n|when\n|when\n|repeat\n|repeat")).unwrap();
}
#[test]
fn short_circuit_uses_control_flow() {
    let ir=compile(&program("when| false and (1 / 0 > 2)\n'x' > out\n|when")).unwrap();
    assert!(ir.contains("logic.rhs"));assert!(ir.contains("br i1"));
}
#[test]
fn vector_access_is_checked() {
    let ir=compile(&program("int a|2|(|1,2|)\na|0| > out")).unwrap();
    assert!(ir.contains("call ptr @sail_vec_at"));
}
#[test]
fn input_pipeline_types_are_checked() {
    compile(&program("int a b\nin > a b\n(a > b) > out")).unwrap();
}
#[test]
fn int_promotes_to_float() {
    let ir=compile(&program("float f\n1 + 2.5 > f\nf > out")).unwrap();
    assert!(ir.contains("sitofp"));
}
#[test]
fn string_assignment_to_int_is_rejected() {
    assert!(compile(&program("int a\n'wrong' > a")).unwrap_err().message.contains("Cannot assign"));
}
#[test]
fn unknown_variable_diagnostic_has_line() {
    let e=compile(&program("missing > out")).unwrap_err();
    assert_eq!(e.line,3);assert!(e.message.contains("Unknown variable"));
}
#[test]
fn unknown_target_is_rejected() {assert!(compile(&program("1 > missing")).is_err());}
#[test]
fn wrong_argument_count_is_rejected() {
    let s="module| T\nfun| int a > F > int\nback| a |back\n|fun\nfun| > main > int\n1 2 > F > out\n|fun\n|module";
    assert!(compile(s).unwrap_err().message.contains("expects 1"));
}
#[test]
fn missing_block_tag_is_rejected() {
    assert!(compile("module| T\nfun| > main > int\n'hello' > out").unwrap_err().message.contains("Missing"));
}
#[test]
fn duplicate_variable_is_rejected() {
    assert!(compile(&program("int a\nint a")).unwrap_err().message.contains("already declared"));
}
#[test]
fn break_outside_loop_is_rejected() {
    assert!(compile(&program("break")).unwrap_err().message.contains("outside a loop"));
}
#[test]
fn unsupported_concurrency_is_explicit() {
    assert!(compile(&program("Multitex tasks||")).unwrap_err().message.contains("not implemented"));
}
#[test]
fn external_links_are_disallowed_in_web() {
    let s=format!("intermodular|\nlink| outside.sl |link\n|intermodular\n{}",program("'hello' > out"));
    assert!(compile(&s).unwrap_err().message.contains("CLI module resolver"));
}
#[test]
fn malformed_string_is_diagnosed() {assert!(compile(&program("'unterminated > out")).is_err());}
#[test]
fn mismatched_vector_type_is_rejected() {
    assert!(compile(&program("int a|2|\n'hello' > a|0|")).is_err());
}
#[test]
fn main_signature_is_enforced() {assert!(compile("fun| > main > float\nback| 0.0 |back\n|fun").is_err());}