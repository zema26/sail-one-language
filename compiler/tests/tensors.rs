use sailc::compile;

fn program(body: &str) -> String {
    format!("module| Spatial\nfun| > main > int\n{body}\n|fun\n|module\n")
}

#[test]
fn matrices_and_high_rank_tensors_generate_checked_llvm() {
    let ir = compile(include_str!("../examples/tensors.sl")).unwrap();
    assert!(ir.contains("call ptr @sail_tensor_new(i64 2"));
    assert!(ir.contains("call ptr @sail_tensor_new(i64 3"));
    assert!(ir.contains("call ptr @sail_tensor_at"));
    assert!(ir.contains("call void @sail_tensor_fill"));
    compile(&program("int t|5, 2, 2, 2, 2, 2|\n3 > t|*|\nt|1, 0, 1, 0, 1| > out")).unwrap();
}

#[test]
fn legacy_vectors_and_rank_one_tensors_are_distinct() {
    let ir = compile(&program("int v|4|\nint t|1, 4|\n2 > v|*|\n3 > t|*|\nv|0| t|0| > out")).unwrap();
    assert!(ir.contains("call ptr @sail_vec_new"));
    assert!(ir.contains("call ptr @sail_tensor_new(i64 1"));
}

#[test]
fn rank_must_be_literal_positive_and_match_shape_count() {
    for body in [
        "int a|0, 2|", "int a|-1, 2|", "int a|2.0, 2, 2|",
        "int r(2)\nint a|r, 2, 2|", "int a|3, 2, 2|",
        "int a|1, 2, 2|", "int a|2, 2,|",
    ] {
        assert!(compile(&program(body)).is_err(), "{body}");
    }
}

#[test]
fn dimensions_and_coordinates_require_integers() {
    for body in [
        "int a|2, 2.5, 2|", "int a|2, true, 2|",
        "int a|2, 2, 2|\na|0, 1.0| > out",
        "int a|2, 2, 2|\n1 > a|false, 1|",
    ] {
        assert!(compile(&program(body)).unwrap_err().message.contains("type int"), "{body}");
    }
}

#[test]
fn coordinate_count_matches_rank_for_reads_and_writes() {
    for body in [
        "int a|2, 2, 2|\na|0| > out",
        "int a|2, 2, 2|\n1 > a|0, 0, 0|",
        "int a|2|\na|0, 1| > out",
    ] {
        assert!(compile(&program(body)).is_err(), "{body}");
    }
}

#[test]
fn broadcast_requires_compatible_scalar_and_collection_destination() {
    for body in [
        "int a\n1 > a|*|", "int a|2, 2, 2|\n'bad' > a|*|",
        "int a|2, 2, 2|\na|*| > out",
        "int a|2, 2, 2|\na > a|*|",
        "int a|2, 2, 2|(|1, 2|)",
    ] {
        assert!(compile(&program(body)).is_err(), "{body}");
    }
}

#[test]
fn scalar_types_and_tensor_class_fields_compile() {
    compile(&program("float f|2, 2, 2|\n1 > f|*|\nbool b|2, 2, 2|\ntrue > b|*|\nchar c|2, 2, 2|\n'λ' > c|*|\nstring s|2, 2, 2|\n'ok' > s|*|")).unwrap();
    compile("module| T\nclass| Layer\nfloat weights|2, 2, 3|\nfun| int unused > Set > int\n2.0 > weights|*|\nback| 0 |back\n|fun\n|class\nfun| > main > int\nLayer layer\n0 > layer.Set\nlayer.weights|1, 2| > out\n|fun\n|module\n").unwrap();
}
