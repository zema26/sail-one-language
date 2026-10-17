use crate::{ast::*, lexer::{Kind,Token}, Error};

pub struct Parser { t: Vec<Token>, p: usize, classes: Vec<String>, depth: usize }
impl Parser {
    pub fn new(t: Vec<Token>) -> Self { Self {t,p:0,classes:Vec::new(),depth:0} }
    fn tok(&self) -> &Token { &self.t[self.p] }
    fn sym(&self,s:&str) -> bool { self.tok().kind == Kind::Sym(s.into()) }
    fn word(&self,s:&str) -> bool { self.tok().kind == Kind::Word(s.into()) }
    fn eat_sym(&mut self,s:&str)->bool { if self.sym(s) {self.p+=1;true} else {false} }
    fn eat_word(&mut self,s:&str)->bool { if self.word(s) {self.p+=1;true} else {false} }
    fn err<T>(&self,s:impl Into<String>)->Result<T,Error> { Err(Error::new(self.tok().line,self.tok().col,s)) }
    fn expect(&mut self,s:&str)->Result<(),Error> { if self.eat_sym(s){Ok(())}else{self.err(format!("Expected '{s}'"))} }
    fn name(&mut self)->Result<String,Error> {
        if let Kind::Word(s)=self.tok().kind.clone(){self.p+=1;Ok(s)}else{self.err("Expected identifier")}
    }
    fn nl(&mut self) { while self.tok().kind==Kind::Newline {self.p+=1;} }
    fn endline(&mut self)->Result<(),Error> {
        if self.tok().kind==Kind::Newline {self.nl();Ok(())}
        else if self.tok().kind==Kind::Eof || self.sym("|"){Ok(())}
        else {self.err("Expected end of statement (newline or semicolon)")}
    }
    fn close(&self,s:&str)->bool {
        self.sym("|") && self.t.get(self.p+1).is_some_and(|t|t.kind==Kind::Word(s.into()))
    }
    fn take_close(&mut self,s:&str)->Result<(),Error> {
        if self.close(s){self.p+=2;Ok(())}else{self.err(format!("Expected |{s}"))}
    }
    fn open(&mut self,s:&str)->Result<(),Error> {
        if !self.eat_word(s){return self.err(format!("Expected {s}|"));}
        self.expect("|")
    }
    fn ty(&mut self)->Result<Ty,Error> {
        if self.eat_sym("&") {Ok(Ty::Generic(self.name()?))}
        else {Ok(Ty::from_name(&self.name()?))}
    }
    fn is_type(&self)->bool {
        match &self.tok().kind {
            Kind::Word(s)=>["int","float","bool","char","string","file"].contains(&s.as_str()) || self.classes.contains(s),
            _=>false,
        }
    }
    pub fn program(mut self)->Result<Program,Error> {
        let mut program=Program::default();
        self.nl();
        self.top(&mut program,None)?;
        Ok(program)
    }
    fn top(&mut self,program:&mut Program,until:Option<&str>)->Result<(),Error> {
        while self.tok().kind!=Kind::Eof {
            self.nl();
            if let Some(s)=until {if self.close(s){return self.take_close(s)}}
            if self.tok().kind==Kind::Eof {break;}
            if self.word("module") {
                self.open("module")?;self.name()?;self.nl();self.top(program,Some("module"))?;
            } else if self.word("comp") || self.word("intermodular") {
                let tag=self.name()?;self.expect("|")?;self.nl();self.top(program,Some(&tag))?;
            } else if self.word("class") {
                self.open("class")?;let name=self.name()?;
                if self.classes.contains(&name){return self.err("Duplicate class");}
                self.classes.push(name.clone());self.nl();
                let mut fields=Vec::new();let mut methods=Vec::new();
                while !self.close("class"){
                    if self.tok().kind==Kind::Eof {return self.err("Missing |class");}
                    if self.word("fun"){methods.push(self.function()?);}
                    else if self.is_type(){fields.extend(self.declarations()?);self.endline()?;}
                    else{return self.err("Expected class field or fun|");}
                    self.nl();
                }
                self.take_close("class")?;
                program.classes.push(Class{name,fields,methods});
            } else if self.word("fun") {
                let f=self.function()?;
                if !f.body.is_empty() || f.name=="main"{program.functions.push(f);}
            } else if self.is_type() {
                program.declarations.extend(self.declarations()?);self.endline()?;
            } else if self.word("link") {
                self.open("link")?;
                let mut path=String::new();
                while !self.close("link") {
                    match self.tok().kind.clone() {
                        Kind::Str(s)|Kind::Word(s)|Kind::Sym(s)=>path.push_str(&s),
                        _=>return self.err("Expected module path followed by |link"),
                    }
                    self.p+=1;
                }
                self.take_close("link")?;
                // Standard IO is intrinsic, no module file required.
                if !["in2out.sln","in2out.sl"].contains(&path.as_str()){program.links.push(path);}
            } else {return self.err("Expected a module declaration, module|, fun|, comp|, class| or intermodular|");}
        }
        if until.is_some(){return self.err(format!("Missing |{}",until.unwrap()));}
        Ok(())
    }
    fn function(&mut self)->Result<Function,Error> {
        let line=self.tok().line;self.open("fun")?;
        let mut params=Vec::new();let mut param_shapes=Vec::new();let mut generic_op=None;
        while !self.sym(">"){
            let ty=self.ty()?;
            if matches!(ty,Ty::Generic(_)) && self.sym("&") {
                if let Ty::Generic(n)=ty {generic_op=Some(n);}
                continue;
            }
            let name=self.name()?;
            let (ty,shape)=self.signature_shape(ty)?;
            params.push((ty,name));
            param_shapes.push(shape);
        }
        self.expect(">")?;let mut name=self.name()?;
        while self.eat_sym("."){name.push('.');name.push_str(&self.name()?);}
        self.expect(">")?;let ret=self.ty()?;
        let (ret,ret_shape)=self.signature_shape(ret)?;
        for dimension in param_shapes.iter().flatten().chain(ret_shape.iter()){
            if !Self::signature_dimension(dimension,&params){
                return self.err("Signature dimensions require nonnegative integer literals, int parameters, or pure integer arithmetic");
            }
        }
        self.nl();let body=self.block(&["fun"])?;self.take_close("fun")?;
        Ok(Function{name,params,ret,param_shapes,ret_shape,body,line,generic_op})
    }
    fn signature_dimension(e:&Expr,params:&[(Ty,String)])->bool{
        match e{
            Expr::Int(n)=>*n>=0,
            Expr::Var(n)=>params.iter().any(|(ty,name)|name==n&&*ty==Ty::Int),
            Expr::Binary(op,a,b) if ["+","-","*","/","%"].contains(&op.as_str())=>
                Self::signature_dimension(a,params)&&Self::signature_dimension(b,params),
            _=>false,
        }
    }
    fn signature_shape(&mut self,ty:Ty)->Result<(Ty,Vec<Expr>),Error>{
        if self.eat_sym("||"){return Ok((Ty::Vector(Box::new(ty)),vec![]));}
        if !self.sym("|") || self.t.get(self.p+1).is_some_and(|t|matches!(&t.kind,Kind::Word(s) if s=="fun")) {
            return Ok((ty,vec![]));
        }
        self.expect("|")?;
        if self.eat_sym("|"){return Ok((Ty::Vector(Box::new(ty)),vec![]));}
        let rank=match self.tok().kind.clone(){
            Kind::Number(n)=>n.parse::<usize>().ok().filter(|n|*n>0&&*n<=128),
            _=>None,
        }.ok_or_else(||Error::new(self.tok().line,self.tok().col,"Tensor signature rank must be an integer literal from 1 to 128"))?;
        self.p+=1;self.expect(",")?;
        let mut shape=vec![self.expr(true,0)?];
        while self.eat_sym(","){shape.push(self.expr(true,0)?);}
        self.expect("|")?;
        if shape.len()!=rank{return self.err(format!("Tensor signature rank {rank} requires {rank} dimensions"));}
        if !matches!(ty,Ty::Int|Ty::Float|Ty::Bool|Ty::Char|Ty::String){
            return self.err("Tensor signatures require scalar element types");
        }
        Ok((Ty::Tensor(Box::new(ty),rank),shape))
    }
    fn block(&mut self,ends:&[&str])->Result<Vec<LocatedStmt>,Error> {
        self.depth+=1;
        if self.depth>128 {return self.err("Block nesting exceeds 128");}
        let mut body=Vec::new();
        self.nl();
        while !ends.iter().any(|s|self.close(s)||(["other","fall"].contains(s) && self.word(s))) {
            if self.tok().kind==Kind::Eof {return self.err(format!("Missing closing block tag |{}",ends[0]));}
            let line=self.tok().line;
            body.push(LocatedStmt{line,stmt:self.statement()?});self.endline()?;
        }
        self.depth-=1;
        Ok(body)
    }
    fn declarations(&mut self)->Result<Vec<Decl>,Error> {
        let mut ty=self.ty()?;let mut declarations=Vec::new();
        loop {
            if self.eat_sym(","){continue;}
            if self.is_type(){ty=self.ty()?;}
            if !matches!(self.tok().kind,Kind::Word(_)){break;}
            let name=self.name()?;let mut d=Decl{ty:ty.clone(),name,init:None,size:None,shape:Vec::new(),values:Vec::new(),file:None};
            if ty==Ty::File {
                self.expect("(")?;
                let path=match self.tok().kind.clone(){Kind::Str(s)=>{self.p+=1;s},_=>return self.err("File name must be a string literal")};
                self.eat_sym(",");let format=self.name()?;self.eat_sym(",");let mut mode=self.name()?;
                if self.eat_sym("+"){mode.push('+');}
                self.expect(")")?;
                d.file=Some((path,format,mode));
            } else {
                if self.eat_sym("|") {
                    let size=self.expr(true,0)?;
                    if self.eat_sym(","){
                        let rank=match size{Expr::Int(n) if n>0=>n as usize,_=>return self.err("Tensor rank must be a positive integer literal")};
                        loop {
                            d.shape.push(self.expr(true,0)?);
                            if !self.eat_sym(","){break;}
                        }
                        if d.shape.len()!=rank{return self.err(format!("Tensor rank {rank} requires {rank} shape dimensions, received {}",d.shape.len()));}
                        d.ty=Ty::Tensor(Box::new(ty.clone()),rank);
                    }else{d.size=Some(size);d.ty=Ty::Vector(Box::new(ty.clone()));}
                    self.expect("|")?;
                } else if self.eat_sym("||") {
                    return self.err("Unbounded task/vector declarations are not supported; specify a size");
                }
                if self.eat_sym("("){
                    if self.eat_sym("|"){
                        while !self.sym("|"){
                            d.values.push(self.expr(false,0)?);
                            if !self.eat_sym(","){break;}
                        }
                        self.expect("|")?;
                    } else {d.init=Some(self.expr(true,0)?);}
                    self.expect(")")?;
                }
            }
            declarations.push(d);
            if matches!(self.tok().kind,Kind::Newline|Kind::Eof)||self.sym("|"){break;}
        }
        if declarations.is_empty(){return self.err("Declaration needs a variable name");}
        Ok(declarations)
    }
    fn statement(&mut self)->Result<Stmt,Error>{
        if self.word("infer"){
            self.open("infer")?;let cond=self.expr(true,0)?;self.endline()?;
            let yes=self.block(&["infer","fall"])?;
            let no=if self.word("fall"){
                self.open("fall")?;self.endline()?;
                self.block(&["infer"])?
            }else{Vec::new()};
            self.take_close("infer")?;return Ok(Stmt::Infer(cond,yes,no));
        }
        if self.word("fall"){return self.err("fall| is only valid inside infer|, before |infer");}
        if self.word("when"){
            self.open("when")?;let cond=self.expr(true,0)?;self.endline()?;
            let yes=self.block(&["when","other"])?;
            let no=if self.word("other"){
                self.open("other")?;self.nl();let b=self.block(&["other","when"])?;
                if self.close("other"){
                    self.take_close("other")?;let after_other=self.p;self.nl();
                    if !self.close("when"){
                        self.p=after_other;return Ok(Stmt::When(cond,yes,b));
                    }
                }b
            }else{Vec::new()};
            self.take_close("when")?;return Ok(Stmt::When(cond,yes,no));
        }
        if self.word("repeat"){
            self.open("repeat")?;let cond=self.expr(true,0)?;self.endline()?;
            let body=self.block(&["repeat"])?;self.take_close("repeat")?;return Ok(Stmt::Repeat(cond,body));
        }
        if self.word("iterate"){
            self.open("iterate")?;
            let ty=if self.is_type(){self.ty()?}else{Ty::Int};
            let name=self.name()?;self.expect("(")?;let init=self.expr(true,0)?;self.expect(")")?;
            let step=if self.eat_sym("++"){1}else if self.eat_sym("--"){-1}else{return self.err("iterate requires ++ or --")};
            let op=match self.tok().kind.clone(){Kind::Sym(s) if ["<",">","<=",">="].contains(&s.as_str())=>{self.p+=1;s},_=>return self.err("iterate requires a comparison bound")};
            let bound=self.expr(true,0)?;self.endline()?;
            let body=self.block(&["iterate"])?;self.take_close("iterate")?;
            return Ok(Stmt::Iterate(Decl{ty,name,init:Some(init),size:None,shape:vec![],values:vec![],file:None},op,bound,step,body));
        }
        if self.word("back"){
            self.open("back")?;let expr=if self.close("back"){None}else{Some(self.expr(true,0)?)};
            self.take_close("back")?;return Ok(Stmt::Return(expr));
        }
        if self.eat_word("break"){return Ok(Stmt::Break);}
        if self.eat_word("continue"){return Ok(Stmt::Continue);}
        if self.is_type(){return Ok(Stmt::Decl(self.declarations()?));}
        if self.word("Multitex"){return self.err("Multitex concurrency is not implemented in this release");}
        let mut stages=vec![self.expr_list()?];
        if self.eat_sym("^>"){
            if stages[0].len()!=1{return self.err("^> requires one state string and one payload string");}
            let state=stages.remove(0).remove(0);
            let payload=self.expr(false,0)?;
            return Ok(Stmt::Escalate(state,payload));
        }
        while self.eat_sym(">"){
            // The proposal's value > |back| shorthand is a terminal pipeline stage.
            if self.close("back"){
                self.take_close("back")?;self.expect("|")?;
                stages.push(vec![Expr::Var("$return".into())]);break;
            }
            stages.push(self.expr_list()?);
        }
        if stages.len()==1 && stages[0].len()==1 {
            return Ok(Stmt::Action(stages.remove(0).remove(0)));
        }
        if stages.len()<2{return self.err("Expected left-to-right pipeline: value > target");}
        Ok(Stmt::Flow(stages))
    }
    fn expr_list(&mut self)->Result<Vec<Expr>,Error>{
        let mut list=Vec::new();
        while !matches!(self.tok().kind,Kind::Newline|Kind::Eof) && !self.sym(">") && !self.sym("^>") && !self.sym("|") {
            if self.eat_sym(","){continue;}
            if self.is_type(){
                let ty=self.ty()?;let name=self.name()?;
                let (ty,shape)=self.signature_shape(ty)?;
                list.push(Expr::Binding(Box::new(Decl{ty,name,init:None,size:None,shape,values:vec![],file:None})));
                continue;
            }
            list.push(self.expr(false,0)?);
        }
        if list.is_empty(){return self.err("Empty pipeline stage");}
        Ok(list)
    }
    fn expr(&mut self,comparison:bool,min:u8)->Result<Expr,Error>{
        self.depth+=1;
        if self.depth>128{return self.err("Expression nesting exceeds 128");}
        let mut left=if self.eat_sym("-"){Expr::Unary("-".into(),Box::new(self.expr(comparison,8)?))}
        else if self.eat_sym("!")||self.eat_word("not"){Expr::Unary("!".into(),Box::new(self.expr(comparison,8)?))}
        else if self.eat_sym("+"){self.expr(comparison,8)?}
        else if self.eat_sym("("){let e=self.expr(true,0)?;self.expect(")")?;e}
        else {
            match self.tok().kind.clone(){
                Kind::Number(n)=>{self.p+=1;if n.contains('.'){Expr::Float(n.parse().map_err(|_|Error::new(self.tok().line,1,"Invalid float"))?)}else{Expr::Int(n.parse().map_err(|_|Error::new(self.tok().line,1,"Integer outside signed 64-bit range"))?)}},
                Kind::Str(s)=>{self.p+=1;Expr::String(s)},
                Kind::Word(s)=>{self.p+=1;match s.as_str(){"true"=>Expr::Bool(true),"false"=>Expr::Bool(false),_=>Expr::Var(s)}},
                _=>return self.err("Expected expression"),
            }
        };
        loop{
            if self.sym("(") && matches!(left,Expr::Var(_)|Expr::Member(_,_)|Expr::Call(_,_)){
                self.p+=1;let mut args=Vec::new();
                if !self.sym(")"){
                    loop{
                        args.push(self.expr(true,0)?);
                        if !self.eat_sym(","){break;}
                    }
                }
                self.expect(")")?;left=Expr::Call(Box::new(left),args);continue;
            }
            if self.sym("."){
                self.p+=1;left=Expr::Member(Box::new(left),self.name()?);continue;
            }
            if self.sym("|") && !matches!(left,Expr::String(_)) {
                // A closing structural tag is never a vector index.
                let is_tag=self.t.get(self.p+1).is_some_and(|t|matches!(&t.kind,Kind::Word(s) if ["back","fun","when","infer","fall","repeat","iterate","other","module","class","comp"].contains(&s.as_str())));
                let next_expr=self.t.get(self.p+1).is_some_and(|t| matches!(&t.kind,Kind::Word(_)|Kind::Number(_)|Kind::Str(_)) || matches!(&t.kind,Kind::Sym(s) if ["(","-","!","*"].contains(&s.as_str())));
                let initializer=self.t.get(self.p+1).is_some_and(|t|t.kind==Kind::Sym("(".into()))
                    && self.t.get(self.p+2).is_some_and(|t|t.kind==Kind::Sym("|".into()));
                if !is_tag && !initializer && next_expr && matches!(left,Expr::Var(_)|Expr::Member(_,_)|Expr::Call(_,_)|Expr::Index(_,_)|Expr::TensorIndex(_,_)){
                    self.p+=1;
                    if self.eat_sym("*"){
                        self.expect("|")?;left=Expr::Broadcast(Box::new(left));continue;
                    }
                    let mut indices=vec![self.expr(true,0)?];
                    while self.eat_sym(","){indices.push(self.expr(true,0)?);}
                    self.expect("|")?;
                    left=if indices.len()==1{Expr::Index(Box::new(left),Box::new(indices.remove(0)))}
                    else{Expr::TensorIndex(Box::new(left),indices)};
                    continue;
                }
            }
            if self.eat_sym("++"){left=Expr::PostInc(Box::new(left),1);continue;}
            if self.eat_sym("--"){left=Expr::PostInc(Box::new(left),-1);continue;}
            let op=match &self.tok().kind{
                Kind::Sym(s)=>s.clone(),
                Kind::Word(s) if ["and","or","Op"].contains(&s.as_str())=>s.clone(),
                _=>break,
            };
            let prec=match op.as_str(){
                "or"|"||"=>1,"and"|"&&"=>2,"=="|"!="=>3,
                "<"|"<="|">="|"~"=>4,">" if comparison=>4,
                "+"|"-"|"Op"=>5,"*"|"/"|"%"=>6,_=>break,
            };
            if prec<min{break;}
            self.p+=1;let right=self.expr(comparison,prec+1)?;
            left=Expr::Binary(op,Box::new(left),Box::new(right));
        }
        self.depth-=1;
        Ok(left)
    }
}