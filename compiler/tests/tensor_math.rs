use sailc::compile;

fn program(body:&str)->String{
    format!("module| T\nfun| > main > int\n{body}\n0 > |back|\n|fun\n|module\n")
}

#[test]
fn shaped_and_dependent_signatures_check_both_boundaries(){
    let ir=compile("module| T\nfun| int n float x|1, n| > Id > float|1, n|\nx > |back|\n|fun\nfun| > main > int\nfloat x|1, 3| y|1, 3|\n3 x > Id > y\n0 > |back|\n|fun\n|module\n").unwrap();
    assert!(ir.matches("call void @sail_tensor_check").count()>=2);
}
#[test]
fn sample_neural_pipeline_compiles(){
    compile(include_str!("../examples/tensor-math.sl")).unwrap();
}
#[test]
fn malformed_or_impure_signature_shapes_are_rejected(){
    for signature in [
        "float x|0, 3|", "float x|2, 3|", "float x|1, 3, 4|",
        "float x|1, 2.5|", "float x|1, unknown|", "int n float x|1, n++|",
        "float n float x|1, n|", "float x|1, -1|",
    ]{
        let source=format!("module| T\nfun| {signature} > F > int\n0 > |back|\n|fun\nfun| > main > int\n0 > |back|\n|fun\n|module\n");
        assert!(compile(&source).is_err(),"{signature}");
    }
}
#[test]
fn tensor_functions_require_returns_on_every_path(){
    assert!(compile("module| T\nfun| bool b > F > float|1, 2|\nfloat t|1, 2|\ninfer| b\nt > |back|\n|infer\n|fun\nfun| > main > int\n0 > |back|\n|fun\n|module\n").unwrap_err().message.contains("every path"));
    compile("module| T\nfun| bool b > F > float|1, 2|\nfloat t|1, 2|\ninfer| b\nt > |back|\nfall|\n'test' ^> 'halt'\n|infer\n|fun\nfun| > main > int\n0 > |back|\n|fun\n|module\n").unwrap();
}
#[test]
fn incompatible_ranks_types_and_math_operands_are_diagnosed(){
    for body in [
        "bool t|1, 2|\nt.sum > out",
        "string t|1, 2|\nt.exp > out",
        "float a|1, 2| b|2, 1, 2|\na + b > a",
        "float a|3, 1, 1, 1|\na * a > a",
        "float a|1, 2|\na % 2 > a",
        "float a|1, 2|\na + true > a",
        "float a|1, 2|\n(a ~ 0.8) > out",
    ]{assert!(compile(&program(body)).is_err(),"{body}");}
}
#[test]
fn partial_reads_reduce_rank_but_partial_writes_are_rejected(){
    compile(&program("int a|3, 2, 2, 2| b|1, 2|\na|0, 1| > b\nb.sum > out")).unwrap();
    assert!(compile(&program("int a|2, 2, 2|\n3 > a|0|")).is_err());
}
