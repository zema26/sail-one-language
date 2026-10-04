use std::{env,fs,path::{Path,PathBuf},process::{Command,ExitCode}};
use sailc::{ast::Program,Error};

fn clang_driver(wasm:bool)->PathBuf {
    for dir in env::split_paths(&env::var_os("PATH").unwrap_or_default()){
        let clang=dir.join("clang");
        if clang.is_file(){
            // Nix's native wrapper injects ELF linker flags that break wasm-ld.
            // Ubuntu's ordinary Clang needs no special handling.
            if wasm{
                if let Some(prefix)=dir.parent(){
                    if let Ok(original)=fs::read_to_string(prefix.join("nix-support/orig-cc")){
                        let unwrapped=Path::new(original.trim()).join("bin/clang");
                        if unwrapped.is_file(){return unwrapped;}
                    }
                }
            }
            return clang;
        }
    }
    PathBuf::from("clang")
}
fn json_string(s:&str)->String {
    let mut out=String::from("\"");
    for c in s.chars(){match c{'"'=>out.push_str("\\\""),'\\'=>out.push_str("\\\\"),'\n'=>out.push_str("\\n"),'\r'=>out.push_str("\\r"),'\t'=>out.push_str("\\t"),c if c<' '=>out.push_str(&format!("\\u{:04x}",c as u32)),c=>out.push(c)}}
    out.push('"');out
}
fn resolve(path:&Path,seen:&mut Vec<PathBuf>,depth:usize)->Result<Program,Error>{
    if depth>32{return Err(Error::new(1,1,"Module nesting exceeds 32"));}
    let canonical=fs::canonicalize(path).map_err(|e|Error::new(1,1,format!("{}: {e}",path.display())))?;
    if seen.contains(&canonical){return Ok(Program::default());}
    seen.push(canonical.clone());
    let source=fs::read_to_string(&canonical).map_err(|e|Error::new(1,1,e.to_string()))?;
    let mut p=sailc::parser::Parser::new(sailc::lexer::lex(&source)?).program()?;
    for link in std::mem::take(&mut p.links){
        let linked=canonical.parent().unwrap().join(link);
        let mut other=resolve(&linked,seen,depth+1)?;
        other.functions.retain(|f|f.name!="main");
        p.functions.extend(other.functions);p.classes.extend(other.classes);
    }
    Ok(p)
}
fn run()->Result<(),Error>{
    let args=env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() || args.contains(&"--help".to_string()){
        println!("sailc 0.1.0 — Rust frontend / LLVM backend\n\
Usage: sailc SOURCE.sl [--emit-llvm|--wasm|--native] [-o OUTPUT] [-O0|-O1|-O2] [--json]\n\
Defaults to LLVM IR on stdout. --wasm creates a sandboxed browser module;\n\
--native links the Sail runtime into an Ubuntu/Linux executable.\n\
Dependencies: Rust (build), Clang 18+ and LLD (code generation).");
        return Ok(());
    }
    if args[0]=="--version"{println!("sailc 0.1.0");return Ok(());}
    let source_path=Path::new(&args[0]);
    let json=args.contains(&"--json".into());
    let p=if json{
        let source=fs::read_to_string(source_path).map_err(|e|Error::new(1,1,e.to_string()))?;
        let p=sailc::parser::Parser::new(sailc::lexer::lex(&source)?).program()?;
        if !p.links.is_empty(){return Err(Error::new(1,1,"Browser compilation does not allow external module links"));}
        p
    }else{resolve(source_path,&mut vec![],0)?};
    let ir=sailc::codegen::generate(&p)?;
    let output=args.iter().position(|a|a=="-o").map(|i|args.get(i+1).cloned().ok_or_else(||Error::new(1,1,"-o requires a path"))).transpose()?;
    let wasm=args.contains(&"--wasm".into());let native=args.contains(&"--native".into());
    if wasm && native{return Err(Error::new(1,1,"Choose --wasm or --native, not both"));}
    if !wasm && !native{
        if let Some(out)=output{fs::write(out,&ir).map_err(|e|Error::new(1,1,e.to_string()))?;}else{print!("{ir}");}
        return Ok(());
    }
    let output=output.unwrap_or_else(||if wasm{"program.wasm".into()}else{"program".into()});
    let ll=PathBuf::from(format!("{output}.ll"));
    fs::write(&ll,&ir).map_err(|e|Error::new(1,1,e.to_string()))?;
    // Embed the runtime so the compiler binary remains portable after installation.
    let runtime=PathBuf::from(format!("{output}.runtime.c"));
    fs::write(&runtime,include_str!("../runtime/runtime.c")).map_err(|e|Error::new(1,1,e.to_string()))?;
    let opt=args.iter().find(|s|["-O0","-O1","-O2"].contains(&s.as_str())).map_or("-O0",String::as_str);
    let mut cmd=Command::new(clang_driver(wasm));
    cmd.args([opt,"-Wno-override-module"]).arg(&ll).arg(&runtime).args(["-o",&output]);
    if wasm{
        cmd.args(["--target=wasm32","-nostdlib","-fno-builtin","-Wl,--no-entry","-Wl,--export=sail_main","-Wl,--export=sail_alloc","-Wl,--export-memory","-Wl,--allow-undefined","-Wl,--max-memory=33554432","-Wl,-z,stack-size=1048576"]);
    }else{cmd.arg("-lm");}
    let result=cmd.output().map_err(|e|Error::new(1,1,format!("Unable to run Clang: {e}. Install clang and lld.")))?;
    let _=fs::remove_file(&runtime);
    if !result.status.success(){return Err(Error::new(1,1,format!("LLVM backend failed: {}",String::from_utf8_lossy(&result.stderr))));}
    if json{println!("{{\"success\":true}}");}else{eprintln!("Compiled {} -> {output}",source_path.display());}
    Ok(())
}
fn main()->ExitCode {
    match run(){Ok(())=>ExitCode::SUCCESS,Err(e)=>{
        if env::args().any(|s|s=="--json"){println!("{{\"success\":false,\"line\":{},\"column\":{},\"message\":{}}}",e.line,e.column,json_string(&e.message));}
        else{eprintln!("error: {e}");}
        ExitCode::FAILURE
    }}
}