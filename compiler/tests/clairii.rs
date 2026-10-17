use sailc::compile;
fn wrap(body:&str)->String{format!("fun| > main > int\n{body}\n0 > |back|\n|fun\n")}

#[test]
fn clairii_example_compiles(){
    compile(include_str!("../examples/clairii.sl")).unwrap();
}
#[test]
fn module_storage_is_shared_and_initialized_at_entry(){
    let ir=compile("int count(3)\nfloat liquid_memory|2, 2, 2|\nfun| > Update > void\ncount + 1 > count\n1.0 > liquid_memory|1, 0|\n|fun\nfun| > main > int\nUpdate()\ncount > out\n0 > |back|\n|fun\n").unwrap();
    assert!(ir.contains("@.sail.global.0 = private global i64"));
    assert!(ir.contains("@.sail.global.1 = private global ptr"));
    assert!(ir.contains("call void @s_Update()"));
}
#[test]
fn outer_products_accept_numeric_vectors_only(){
    compile(&wrap("int a|1, 2|\nfloat b|1, 3| c|2, 2, 3|\na.cross(b) * 0.01 > c")).unwrap();
    for body in [
        "float a|2, 2, 2|\na.cross(a) > out",
        "bool a|1, 2|\na.cross(a) > out",
        "float a|1, 2|\na.cross() > out",
        "float a|1, 2|\na.cross(3) > out",
        "float a|1, 2|\na.cross(a, a) > out",
    ]{assert!(compile(&wrap(body)).is_err(),"{body}");}
}
#[test]
fn typed_destinations_calls_and_append_are_checked(){
    compile(&wrap("string s('a')\ns.append('b')\ntrue > bool valid\n1 > int n\ns.append(s) > string result\nresult > out\nin > float score")).unwrap();
    for body in [
        "'x' > int n", "true > bool n\nfalse > bool n",
        "string s\ns.append(1)", "string s\ns.append()",
        "'x'.append('y')", "1 > out\nint n(0)\nn.verify_axioms > out",
        "true > bool b\nwhen| b\n1 > int inner\n|when\ninner > out",
    ]{assert!(compile(&wrap(body)).is_err(),"{body}");}
}
#[test]
fn string_vector_defaults_are_valid_append_receivers(){
    let ir=compile(&wrap("string words|2|\nwords|0|.append('x')\nwords|1| > out")).unwrap();
    assert!(ir.contains("call void @sail_vec_fill"));
}
#[test]
fn namespaces_and_explicit_tensor_policies_are_normal_functions(){
    let source=format!("fun| int n > Tokenizer.encode > int\nn + 1 > |back|\n|fun\nfun| float a|1, 2| > Tensor.confidence > float\na.max > |back|\n|fun\n{}",
        wrap("float a|1, 2|\nTokenizer.encode(2) > int n\nn > Tokenizer.encode > n\na.confidence > float c\na.confidence() > c"));
    compile(&source).unwrap();
    let error=compile(&wrap("float a|1, 2|\na.verify_axioms > out")).unwrap_err();
    assert!(error.message.contains("explicit Sail function"));
}
#[test]
fn unsafe_or_ambiguous_module_initializers_are_rejected(){
    for data in ["int a(unknown)", "int a(b)\nint b(1)", "int a(a)", "int a(1)\nint a(2)", "int a(1)\nint b(a++)", "int a(Make())"]{
        assert!(compile(&format!("{data}\n{}",wrap(""))).is_err(),"{data}");
    }
    compile(&format!("int n(2)\nfloat a|2, n, n|\nint v|n|(|3, 7|)\n{}",wrap("a.length v|1| > out"))).unwrap();
}
#[test]
fn alternate_and_legacy_other_endings_both_compile(){
    for ending in ["|other", "|other\n|when", "|when"]{
        compile(&wrap(&format!("when| true\n1 > out\nother|\n2 > out\n{ending}"))).unwrap();
    }
    compile(&wrap("when| true\nwhen| false\n1 > out\nother|\n2 > out\n|other\nother|\n3 > out\n|other\n4 > out")).unwrap();
}
#[test]
fn namespace_symbols_do_not_collide_with_underscores(){
    compile(&format!("fun| > A.b > int\n1 > |back|\n|fun\nfun| > A_b > int\n2 > |back|\n|fun\n{}",wrap("A.b() A_b() > out"))).unwrap();
}
