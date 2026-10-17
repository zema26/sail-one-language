use sailc::compile;

fn program(body: &str) -> String {
    format!("module| Test\nfun| > main > int\n{body}\n0 > |back|\n|fun\n|module\n")
}

#[test]
fn fuzzy_operator_is_an_inclusive_numeric_comparison() {
    let ir = compile(&program("bool accepted(0.85 ~ 0.90)\naccepted > out\n(3 ~ 2) > out")).unwrap();
    assert!(ir.contains("fcmp oge double"));
    assert!(ir.contains("icmp sge i64"));
}

#[test]
fn infer_fall_nests_with_when_loops_and_optional_fallbacks() {
    compile(&program("int n(0)\ninfer| 0.9 ~ 0.8\ninfer| true\nn++\nfall|\n0 > n\n|infer\nfall|\nwhen| true\n1 > n\n|when\n|infer\nrepeat| false\ninfer| false\nbreak\nfall|\ncontinue\n|infer\n|repeat")).unwrap();
}

#[test]
fn fuzzy_operands_must_be_numeric_scalars() {
    for expression in ["true ~ 0.5", "0.5 ~ false", "'a' ~ 'b'", "v ~ 0.5", "0.5 ~ v"] {
        let source = program(&format!("float v|1, 2|\n({expression}) > out"));
        let error = compile(&source).unwrap_err();
        assert!(error.message.contains("~ requires int or float"), "{expression}: {}", error.message);
    }
}

#[test]
fn infer_does_not_silently_coerce_numeric_or_string_conditions() {
    for condition in ["0.85", "1", "'yes'"] {
        let error = compile(&program(&format!("infer| {condition}\n|infer"))).unwrap_err();
        assert!(error.message.contains("infer| requires a bool"), "{}", error.message);
    }
}

#[test]
fn escalation_requires_two_strings_and_terminates_its_block() {
    let ir = compile(&program("'TEMPORAL_LOCK' ^> 'candidate'\n'UNREACHABLE' > out")).unwrap();
    assert!(ir.contains("call void @sail_escalate"));
    assert!(ir.contains("unreachable"));
    assert!(!ir.contains("UNREACHABLE"));
    for body in ["1 ^> 'token'", "'LOCK' ^> 2", "'LOCK' 'other' ^> 'token'", "'LOCK' ^>", "'LOCK' ^> 'token' > out"] {
        assert!(compile(&program(body)).is_err(), "{body}");
    }
}

#[test]
fn malformed_infer_and_fall_report_source_diagnostics() {
    for body in [
        "fall|\n'bad' > out",
        "infer| true\n'bad' > out",
        "infer| true\nfall|\nfall|\n|infer",
        "infer| true\nother|\n|infer",
        "infer| true\nfall|\n|fall\n|infer",
    ] {
        let error = compile(&program(body)).unwrap_err();
        assert!(error.line > 0);
    }
}

#[test]
fn branch_local_declarations_do_not_escape() {
    let error = compile(&program("infer| true\nint local(1)\nfall|\nint local(2)\n|infer\nlocal > out")).unwrap_err();
    assert!(error.message.contains("local"));
}

#[test]
fn supplied_branch_and_adapted_decoder_examples_compile() {
    compile(include_str!("../examples/neuro-symbolic.sl")).unwrap();
    compile(include_str!("../examples/token-decoder.sl")).unwrap();
}

#[test]
fn return_shorthand_accepts_function_pipelines_but_not_multiple_values() {
    compile("module| T\nfun| int n > Id > int\nn > |back|\n|fun\nfun| > main > int\n3 > Id > |back|\n|fun\n|module\n").unwrap();
    assert!(compile(&program("1 2 > |back|")).unwrap_err().message.contains("exactly one"));
}
