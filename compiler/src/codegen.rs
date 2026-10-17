use std::collections::{HashMap, HashSet};
use crate::{ast::*, Error};
mod tensor;
mod clair;

#[derive(Clone, Debug)]
struct Value { ty: Ty, ir: String }
#[derive(Clone)]
struct Slot { ty: Ty, ptr: String }
#[derive(Clone)]
struct Sig { params: Vec<Ty>, ret: Ty, symbol: String }
struct Generator<'a> {
    program: &'a Program, signatures: HashMap<String,Sig>, globals: Vec<String>,
    strings: HashMap<String,String>, functions: Vec<String>,
    module_slots: HashMap<String,Slot>,
}
struct FunctionGen<'a,'b> {
    gen: &'a mut Generator<'b>, code: Vec<String>, allocas: Vec<String>,
    vars: Vec<HashMap<String,Slot>>, counter: usize, label_counter: usize,
    terminated: bool, loops: Vec<(String,String)>, line: usize, ret: Ty,
    generic_ty: Option<Ty>, generic_op: Option<String>,
    ret_contract: Option<(Value,usize)>,
}
fn symbol(s:&str)->String { format!("s_{}",s.replace('_',"_u").replace('.',"_d")) }
fn op_name(op:&str)->&str {match op {"+"=>"add","-"=>"sub","*"=>"mul","/"=>"div","%"=>"mod",_=>"unknown"}}
fn ty_name(ty:&Ty)->&str {match ty {Ty::Float=>"float",Ty::String=>"string",_=>"int"}}
pub fn generate(program:&Program)->Result<String,Error>{
    let mut g=Generator{program,signatures:HashMap::new(),globals:vec![],strings:HashMap::new(),functions:vec![],module_slots:HashMap::new()};
    let mut names=HashSet::new();
    for f in &program.functions {
        if !names.insert(f.name.clone()){return Err(Error::new(f.line,1,format!("Duplicate function '{}'",f.name)));}
        g.add_signature(f,None)?;
    }
    for c in &program.classes{
        let mut fields=HashSet::new();
        for f in &c.fields {
            if !fields.insert(&f.name){return Err(Error::new(1,1,format!("Duplicate field {}.{}",c.name,f.name)));}
        }
        for f in &c.methods{g.add_signature(f,Some(&c.name))?;}
    }
    g.prepare_module_data()?;
    let main=program.functions.iter().find(|f|f.name=="main").ok_or_else(||Error::new(1,1,"Missing main"))?;
    if main.ret!=Ty::Int{return Err(Error::new(main.line,1,"main must return int"));}
    if !(main.params.is_empty() || main.params.len()==1 && main.params[0].0==Ty::Vector(Box::new(Ty::String))){
        return Err(Error::new(main.line,1,"main accepts no parameters or string args| |"));
    }
    for f in &program.functions {g.emit_variants(f,None)?;}
    for c in &program.classes {for f in &c.methods{g.emit_variants(f,Some(c))?;}}
    let mut ir=String::from("; Sail compiler 0.1.1 — typed LLVM IR\nsource_filename = \"program.sl\"\n\n");
    for c in &program.classes {
        ir.push_str(&format!("%class.{} = type {{ {} }}\n",c.name,c.fields.iter().map(|d|d.ty.llvm()).collect::<Vec<_>>().join(", ")));
    }
    ir.push_str(RUNTIME_DECLS);
    ir.push('\n');ir.push_str(&g.globals.join("\n"));ir.push_str("\n\n");ir.push_str(&g.functions.join("\n\n"));ir.push('\n');
    Ok(ir)
}
impl Generator<'_> {
    fn add_signature(&mut self,f:&Function,class:Option<&str>)->Result<(),Error>{
        let name=class.map_or(f.name.clone(),|c|format!("{c}.{}",f.name));
        let mut params=f.params.iter().map(|p|p.0.clone()).collect::<Vec<_>>();
        if let Some(c)=class{params.insert(0,Ty::Object(c.to_owned()));}
        if f.name=="main" && class.is_none(){params.clear();}
        let sig=Sig{params,ret:f.ret.clone(),symbol:if name=="main"{"sail_main".into()}else{symbol(&name)}};
        if self.signatures.insert(name.clone(),sig).is_some(){return Err(Error::new(f.line,1,format!("Duplicate function {name}")));}
        Ok(())
    }
    fn string(&mut self,s:&str)->String{
        if let Some(name)=self.strings.get(s){return name.clone();}
        let name=format!("@.str{}",self.strings.len());
        let escaped=s.as_bytes().iter().map(|b|format!("\\{b:02X}")).collect::<String>();
        self.globals.push(format!("{name} = private unnamed_addr constant [{} x i8] c\"{escaped}\\00\"",s.len()+1));
        self.strings.insert(s.to_owned(),name.clone());name
    }
    fn emit_variants(&mut self,f:&Function,class:Option<&Class>)->Result<(),Error>{
        if f.params.iter().any(|p|matches!(p.0,Ty::Generic(_))){
            if class.is_some(){return Err(Error::new(f.line,1,"Generic member functions are not yet supported"));}
            if f.generic_op.is_some(){
                for ty in [Ty::Int,Ty::Float] {
                    for op in ["+","-","*","/","%"]{self.emit_function(f,None,Some(ty.clone()),Some(op.into()))?;}
                }
            }else{
                for ty in [Ty::Int,Ty::Float,Ty::String]{self.emit_function(f,None,Some(ty),None)?;}
            }
        }else{self.emit_function(f,class,None,None)?;}
        Ok(())
    }
    fn emit_function(&mut self,f:&Function,class:Option<&Class>,generic_ty:Option<Ty>,generic_op:Option<String>)->Result<(),Error>{
        let name=class.map_or(f.name.clone(),|c|format!("{}.{}",c.name,f.name));
        let sig=self.signatures[&name].clone();
        let specialize=|t:&Ty|if matches!(t,Ty::Generic(_)){generic_ty.clone().unwrap_or(t.clone())}else{t.clone()};
        let ret=specialize(&f.ret);
        let mut symbol=sig.symbol;
        if let Some(ty)=&generic_ty{
            symbol.push_str(&format!("__{}",ty_name(ty)));
            if let Some(op)=&generic_op{symbol.push_str(&format!("_{}",op_name(op)));}
        }
        let mut fg=FunctionGen{gen:self,code:vec![],allocas:vec![],vars:vec![HashMap::new()],counter:0,label_counter:0,terminated:false,loops:vec![],line:f.line,ret:ret.clone(),generic_ty,generic_op,ret_contract:None};
        let mut args=Vec::new();
        let is_main=name=="main";
        if is_main {
            for d in fg.gen.program.declarations.clone(){
                let slot=fg.gen.module_slots[&d.name].clone();
                fg.initialize(&d,&slot)?;
            }
        }
        if let Some(c)=class {
            args.push("ptr %self".to_owned());
            fg.vars[0].insert("self".into(),Slot{ty:Ty::Object(c.name.clone()),ptr:"%self.slot".into()});
            fg.allocas.push("%self.slot = alloca ptr".into());fg.emit("store ptr %self, ptr %self.slot");
            for (i,d) in c.fields.iter().enumerate(){
                let p=fg.tmp();fg.emit(format!("{p} = getelementptr %class.{}, ptr %self, i32 0, i32 {i}",c.name));
                fg.vars[0].insert(d.name.clone(),Slot{ty:d.ty.clone(),ptr:p});
            }
        }
        for (i,(ty,n)) in f.params.iter().enumerate(){
            let ty=fg.resolve_ty(ty);
            if is_main{
                let p=fg.alloc_slot(n,ty.clone(),true)?;
                let z=fg.zero(&ty)?;
                fg.store(&p,z)?;
            }else{
                args.push(format!("{} %arg{i}",ty.llvm()));
                // Parameters intentionally shadow fields.
                let p=fg.alloc_slot(n,ty.clone(),true)?;
                fg.emit(format!("store {} %arg{i}, ptr {}",ty.llvm(),p.ptr));
            }
        }
        for (i,(_,name)) in f.params.iter().enumerate(){
            if !f.param_shapes[i].is_empty(){
                let slot=fg.lookup(name).unwrap();let value=fg.load(&slot);
                fg.check_tensor_shape(value,&f.param_shapes[i])?;
            }
        }
        if !f.ret_shape.is_empty(){
            let dims=fg.int_array(&f.ret_shape)?;
            fg.ret_contract=Some((dims,f.ret_shape.len()));
        }
        fg.body(&f.body,false)?;
        if !fg.terminated{
            if matches!(ret,Ty::Tensor(..)){return Err(Error::new(f.line,1,"Tensor-returning functions must return a tensor on every path"));}
            if ret==Ty::Void{fg.emit("ret void");}
            else {let z=fg.zero(&ret)?;fg.emit(format!("ret {} {}",ret.llvm(),z.ir));}
        }
        let mut text=format!("define {} @{symbol}({}) {{\nentry:\n",ret.llvm(),args.join(", "));
        for a in &fg.allocas{text.push_str(&format!("  {a}\n"));}
        for c in &fg.code{text.push_str(&format!("  {c}\n"));}
        text.push('}');
        self.functions.push(text);
        Ok(())
    }
}
impl FunctionGen<'_,'_> {
    fn err<T>(&self,s:impl Into<String>)->Result<T,Error>{Err(Error::new(self.line,1,s))}
    fn emit(&mut self,s:impl Into<String>){self.code.push(s.into());}
    fn tmp(&mut self)->String{let s=format!("%v{}",self.counter);self.counter+=1;s}
    fn label(&mut self,prefix:&str)->String{let s=format!("{prefix}{}",self.label_counter);self.label_counter+=1;s}
    fn mark(&mut self,label:&str){self.emit(format!("{label}:"));self.terminated=false;}
    fn branch(&mut self,label:&str){if !self.terminated{self.emit(format!("br label %{label}"));self.terminated=true;}}
    fn resolve_ty(&self,ty:&Ty)->Ty{match ty{Ty::Generic(_)=>self.generic_ty.clone().unwrap_or(ty.clone()),Ty::Vector(t)=>Ty::Vector(Box::new(self.resolve_ty(t))),Ty::Tensor(t,r)=>Ty::Tensor(Box::new(self.resolve_ty(t)),*r),_=>ty.clone()}}
    fn lookup(&self,n:&str)->Option<Slot>{self.vars.iter().rev().find_map(|v|v.get(n).cloned()).or_else(||self.gen.module_slots.get(n).cloned())}
    fn alloc_slot(&mut self,n:&str,ty:Ty,shadow:bool)->Result<Slot,Error>{
        if !shadow && self.vars.last().unwrap().contains_key(n){return self.err(format!("Variable '{n}' is already declared in this scope"));}
        let ptr=self.tmp();self.allocas.push(format!("{ptr} = alloca {}",ty.llvm()));
        let slot=Slot{ty,ptr};self.vars.last_mut().unwrap().insert(n.into(),slot.clone());Ok(slot)
    }
    fn load(&mut self,slot:&Slot)->Value{
        let ir=self.tmp();self.emit(format!("{ir} = load {}, ptr {}",slot.ty.llvm(),slot.ptr));
        Value{ty:slot.ty.clone(),ir}
    }
    fn store(&mut self,slot:&Slot,value:Value)->Result<(),Error>{
        let value=self.cast(value,&slot.ty)?;
        self.emit(format!("store {} {}, ptr {}",slot.ty.llvm(),value.ir,slot.ptr));Ok(())
    }
    fn call_rt(&mut self,n:&str,ret:Ty,args:Vec<Value>)->Value{
        let a=args.iter().map(|a|format!("{} {}",a.ty.llvm(),a.ir)).collect::<Vec<_>>().join(", ");
        if ret==Ty::Void{self.emit(format!("call void @{n}({a})"));Value{ty:Ty::Void,ir:String::new()}}
        else{let ir=self.tmp();self.emit(format!("{ir} = call {} @{n}({a})",ret.llvm()));Value{ty:ret,ir}}
    }
    fn int(i:i64)->Value{Value{ty:Ty::Int,ir:i.to_string()}}
    fn zero(&mut self,ty:&Ty)->Result<Value,Error>{
        Ok(match ty{
            Ty::Int|Ty::Char=>Value{ty:ty.clone(),ir:"0".into()},
            Ty::Bool=>Value{ty:Ty::Bool,ir:"false".into()},
            Ty::Float=>Value{ty:Ty::Float,ir:"0.0".into()},
            Ty::String=>Value{ty:Ty::String,ir:self.gen.string("")},
            Ty::Vector(_)=>self.new_vector(ty,Self::int(0))?,
            Ty::File=>Value{ty:Ty::File,ir:"null".into()},
            Ty::Object(name)=>{
                let class=self.gen.program.classes.iter().find(|c|c.name==*name).cloned().ok_or_else(||Error::new(self.line,1,format!("Unknown class {name}")))?;
                let v=self.call_rt("sail_alloc",ty.clone(),vec![Self::int((class.fields.len().max(1)*8) as i64)]);
                for (i,d) in class.fields.iter().enumerate(){
                    if matches!(d.ty,Ty::Object(_)){return self.err("Object-valued fields are not supported in this release");}
                    let p=self.tmp();self.emit(format!("{p} = getelementptr %class.{name}, ptr {}, i32 0, i32 {i}",v.ir));
                    let slot=Slot{ty:d.ty.clone(),ptr:p};
                    let init=if !d.shape.is_empty(){self.new_tensor(d,&d.ty)?}
                    else if let Some(e)=&d.init{self.expr(e)?}else if let Some(size)=&d.size{let n=self.expr(size)?;let n=self.cast(n,&Ty::Int)?;self.new_vector(&d.ty,n)?}else{self.zero(&d.ty)?};
                    self.store(&slot,init)?;
                }
                v
            },
            _=>return self.err("Unresolved generic type or void variable"),
        })
    }
    fn cast(&mut self,v:Value,ty:&Ty)->Result<Value,Error>{
        if &v.ty==ty{return Ok(v);}
        if (v.ty==Ty::Char && *ty==Ty::Int)||(v.ty==Ty::Int && *ty==Ty::Char){return Ok(Value{ty:ty.clone(),ir:v.ir});}
        if *ty==Ty::Char && v.ty==Ty::String{
            return Ok(self.call_rt("sail_to_char",Ty::Char,vec![v]));
        }
        let ir=self.tmp();
        let op=match (&v.ty,ty){
            (Ty::Int|Ty::Char,Ty::Float)=>"sitofp",
            (Ty::Bool,Ty::Int|Ty::Char)=>"zext",
            (Ty::Int|Ty::Char,Ty::Bool)=>{self.emit(format!("{ir} = icmp ne i64 {}, 0",v.ir));return Ok(Value{ty:Ty::Bool,ir});},
            (Ty::Float,Ty::Bool)=>{self.emit(format!("{ir} = fcmp une double {}, 0.0",v.ir));return Ok(Value{ty:Ty::Bool,ir});},
            (Ty::String,Ty::Bool)=>{let len=self.call_rt("sail_strlen",Ty::Int,vec![v]);return self.cast(len,ty);},
            (Ty::Bool,Ty::Float)=>{let i=self.cast(v,&Ty::Int)?;return self.cast(i,ty);},
            _=>return self.err(format!("Cannot assign {:?} to {:?}",v.ty,ty)),
        };
        self.emit(format!("{ir} = {op} {} {} to {}",v.ty.llvm(),v.ir,ty.llvm()));Ok(Value{ty:ty.clone(),ir})
    }
    fn slot(&mut self,e:&Expr)->Result<Slot,Error>{
        match e{
            Expr::Var(n)=>self.lookup(n).ok_or_else(||Error::new(self.line,1,format!("Unknown variable '{n}'"))),
            Expr::Index(base,index)=>{
                let base_value=self.expr(base)?;
                if matches!(base_value.ty,Ty::Tensor(_,_)){
                    return self.tensor_slot(base_value,std::slice::from_ref(index.as_ref()));
                }
                let base=base_value;let item=match &base.ty{Ty::Vector(t)=>*t.clone(),Ty::String=>Ty::Char,_=>return self.err("Indexing requires a vector, tensor or string")};
                let index=self.expr(index)?;let index=self.cast(index,&Ty::Int)?;
                if base.ty==Ty::String{return self.err("Strings are immutable; indexed reads are supported, not indexed writes");}
                let p=self.call_rt("sail_vec_at",Ty::File,vec![base,index]);
                Ok(Slot{ty:item,ptr:p.ir})
            },
            Expr::TensorIndex(base,indices)=>{
                let base=self.expr(base)?;self.tensor_slot(base,indices)
            },
            Expr::Member(base,n)=>{
                let b=self.expr(base)?;
                let class=if let Ty::Object(c)=&b.ty{c.clone()}else{return self.err("Member assignment requires a class instance")};
                let c=self.gen.program.classes.iter().find(|c|c.name==class).unwrap();
                let (i,d)=c.fields.iter().enumerate().find(|(_,d)|d.name==*n).ok_or_else(||Error::new(self.line,1,format!("Unknown member {class}.{n}")))?;
                let ty=d.ty.clone();let p=self.tmp();self.emit(format!("{p} = getelementptr %class.{class}, ptr {}, i32 0, i32 {i}",b.ir));
                Ok(Slot{ty,ptr:p})
            },
            _=>self.err("Pipeline destination must be a variable, vector element, field, function or out"),
        }
    }
    // Stack scratch arrays live in the entry block, including accesses in loops.
    fn int_array(&mut self,expressions:&[Expr])->Result<Value,Error>{
        let ptr=self.tmp();
        self.allocas.push(format!("{ptr} = alloca [{} x i64]",expressions.len()));
        for (i,e) in expressions.iter().enumerate(){
            let value=self.expr(e)?;
            if value.ty!=Ty::Int{return self.err("Tensor dimensions and coordinates must have type int");}
            let cell=self.tmp();
            self.emit(format!("{cell} = getelementptr i64, ptr {ptr}, i64 {i}"));
            self.emit(format!("store i64 {}, ptr {cell}",value.ir));
        }
        Ok(Value{ty:Ty::File,ir:ptr})
    }
    fn tensor_slot(&mut self,base:Value,indices:&[Expr])->Result<Slot,Error>{
        let (item,rank)=match &base.ty{
            Ty::Tensor(item,rank)=>(*item.clone(),*rank),
            _=>return self.err("Multiple coordinates require a tensor"),
        };
        if indices.len()!=rank{return self.err(format!("Tensor rank {rank} requires {rank} coordinates, received {}",indices.len()));}
        let coordinates=self.int_array(indices)?;
        let ptr=self.call_rt("sail_tensor_at",Ty::File,vec![base,Self::int(rank as i64),coordinates]);
        Ok(Slot{ty:item,ptr:ptr.ir})
    }
    fn new_tensor(&mut self,d:&Decl,ty:&Ty)->Result<Value,Error>{
        if d.init.is_some() || !d.values.is_empty(){return self.err("Initialize tensors with scalar > name|*| or indexed assignments");}
        let (item,rank)=match ty{Ty::Tensor(item,rank)=>(*item.clone(),*rank),_=>return self.err("Tensor declaration requires a ranked type")};
        if !matches!(item,Ty::Int|Ty::Float|Ty::Bool|Ty::Char|Ty::String){return self.err("Tensor elements must have a scalar type");}
        let shape=self.int_array(&d.shape)?;
        let tensor=self.call_rt("sail_tensor_new",ty.clone(),vec![Self::int(rank as i64),shape]);
        // String defaults must be valid empty strings, not null pointers.
        if item==Ty::String{
            let empty=self.zero(&item)?;self.fill_collection(tensor.clone(),empty)?;
        }
        Ok(tensor)
    }
    fn new_vector(&mut self,ty:&Ty,size:Value)->Result<Value,Error>{
        let vector=self.call_rt("sail_vec_new",ty.clone(),vec![size]);
        if matches!(ty,Ty::Vector(item) if **item==Ty::String){
            let empty=self.zero(&Ty::String)?;
            self.fill_collection(vector.clone(),empty)?;
        }
        Ok(vector)
    }
    fn collection_item(&self,ty:&Ty)->Result<Ty,Error>{
        match ty{Ty::Tensor(t,_)|Ty::Vector(t)=>Ok(*t.clone()),_=>self.err("Broadcast assignment requires a vector or tensor")}
    }
    fn fill_collection(&mut self,collection:Value,value:Value)->Result<Value,Error>{
        let item=self.collection_item(&collection.ty)?;
        if !matches!(item,Ty::Int|Ty::Float|Ty::Bool|Ty::Char|Ty::String){return self.err("Broadcast values must have a scalar type");}
        let value=self.cast(value,&item)?;
        let bits=match value.ty{
            Ty::Float=>{let ir=self.tmp();self.emit(format!("{ir} = bitcast double {} to i64",value.ir));Value{ty:Ty::Int,ir}},
            Ty::String=>{let ir=self.tmp();self.emit(format!("{ir} = ptrtoint ptr {} to i64",value.ir));Value{ty:Ty::Int,ir}},
            Ty::Bool=>self.cast(value.clone(),&Ty::Int)?,
            _=>Value{ty:Ty::Int,ir:value.ir.clone()},
        };
        self.call_rt(if matches!(collection.ty,Ty::Tensor(_,_)){"sail_tensor_fill"}else{"sail_vec_fill"},Ty::Void,vec![collection,bits]);
        Ok(value)
    }
    fn expr(&mut self,e:&Expr)->Result<Value,Error>{
        match e{
            Expr::Int(i)=>Ok(Self::int(*i)),
            Expr::Float(f)=>Ok(Value{ty:Ty::Float,ir:format!("0x{:016X}",f.to_bits())}),
            Expr::Bool(b)=>Ok(Value{ty:Ty::Bool,ir:b.to_string()}),
            Expr::String(s)=>Ok(Value{ty:Ty::String,ir:self.gen.string(s)}),
            Expr::Call(target,args)=>self.call_expression(target,args),
            Expr::Binding(_)=>self.err("Typed pipeline declarations are only valid as destinations"),
            Expr::Var(_)=>{let s=self.slot(e)?;Ok(self.load(&s))},
            Expr::Index(base,index)=>{
                let b=self.expr(base)?;
                if b.ty==Ty::String{let i=self.expr(index)?;let i=self.cast(i,&Ty::Int)?;Ok(self.call_rt("sail_string_at",Ty::Char,vec![b,i]))}
                else if matches!(b.ty,Ty::Tensor(..)){self.tensor_read(b,std::slice::from_ref(index.as_ref()))}
                else{let s=self.slot(e)?;Ok(self.load(&s))}
            },
            Expr::TensorIndex(base,indices)=>{let b=self.expr(base)?;self.tensor_read(b,indices)},
            Expr::Broadcast(_)=>self.err("Broadcast wildcard |*| is only valid as an assignment destination"),
            Expr::Member(base,n)=>{
                let b=self.expr(base)?;
                if matches!(b.ty,Ty::Tensor(..)) && ["exp","sum","max","argmax"].contains(&n.as_str()){
                    return self.tensor_member(b,n);
                }
                if matches!(b.ty,Ty::Tensor(..)) && n!="length"{
                    return self.tensor_extension(n,b,vec![]);
                }
                if ["verify_axioms","confidence"].contains(&n.as_str()){
                    return self.err(format!("{n} requires a ranked tensor receiver and an explicit Tensor.{n} function; a scalar float is not a thought vector"));
                }
                if n=="length" && matches!(b.ty,Ty::Vector(_)|Ty::Tensor(_,_)|Ty::String){
                    if b.ty==Ty::String{Ok(self.call_rt("sail_strlen",Ty::Int,vec![b]))}
                    else{let p=self.tmp();self.emit(format!("{p} = load i64, ptr {}",b.ir));Ok(Value{ty:Ty::Int,ir:p})}
                }else if n=="eof" && b.ty==Ty::File {Ok(self.call_rt("sail_file_eof",Ty::Bool,vec![b]))}
                else{let s=self.slot(e)?;Ok(self.load(&s))}
            },
            Expr::PostInc(e,delta)=>{
                let s=self.slot(e)?;if s.ty!=Ty::Int{return self.err("++/-- require int");}
                let old=self.load(&s);let new=self.binary("+",old.clone(),Self::int(*delta))?;self.store(&s,new)?;Ok(old)
            },
            Expr::Unary(op,e)=>{
                let v=self.expr(e)?;
                if op=="-"&&matches!(v.ty,Ty::Tensor(..)){return self.tensor_binary("-",Self::int(0),v);}
                if op=="!"{let v=self.cast(v,&Ty::Bool)?;let ir=self.tmp();self.emit(format!("{ir} = xor i1 {}, true",v.ir));Ok(Value{ty:Ty::Bool,ir})}
                else if v.ty==Ty::Float{let ir=self.tmp();self.emit(format!("{ir} = fneg double {}",v.ir));Ok(Value{ty:Ty::Float,ir})}
                else{let v=self.cast(v,&Ty::Int)?;self.binary("-",Self::int(0),v)}
            },
            Expr::Binary(op,a,b)=>{
                let op=if op=="Op"{self.generic_op.clone().ok_or_else(||Error::new(self.line,1,"Op is only valid in &Op generic functions"))?}else{op.clone()};
                if ["&&","||","and","or"].contains(&op.as_str()){return self.logical(&op,a,b);}
                let a=self.expr(a)?;let b=self.expr(b)?;self.binary(&op,a,b)
            },
        }
    }
    fn logical(&mut self,op:&str,a:&Expr,b:&Expr)->Result<Value,Error>{
        // Use a local slot so nested short-circuit expressions need no predecessor bookkeeping.
        let p=self.tmp();self.allocas.push(format!("{p} = alloca i1"));
        let a=self.expr(a)?;let a=self.cast(a,&Ty::Bool)?;self.emit(format!("store i1 {}, ptr {p}",a.ir));
        let rhs=self.label("logic.rhs");let end=self.label("logic.end");
        let and=op=="&&"||op=="and";
        self.emit(format!("br i1 {}, label %{}, label %{}",a.ir,if and{&rhs}else{&end},if and{&end}else{&rhs}));
        self.mark(&rhs);let b=self.expr(b)?;let b=self.cast(b,&Ty::Bool)?;self.emit(format!("store i1 {}, ptr {p}",b.ir));self.branch(&end);
        self.mark(&end);Ok(self.load(&Slot{ty:Ty::Bool,ptr:p}))
    }
    fn binary(&mut self,op:&str,a:Value,b:Value)->Result<Value,Error>{
        if op!="~" && (matches!(a.ty,Ty::Tensor(..))||matches!(b.ty,Ty::Tensor(..))){
            return self.tensor_binary(op,a,b);
        }
        if op=="~"{
            if !matches!(a.ty,Ty::Int|Ty::Float)||!matches!(b.ty,Ty::Int|Ty::Float){
                return self.err("Fuzzy operator ~ requires int or float scalar operands; index or reduce tensors explicitly");
            }
            // Inclusive confidence threshold; ordered floating comparison rejects NaN.
            return self.binary(">=",a,b);
        }
        if a.ty==Ty::String && b.ty==Ty::String{
            return match op{
                "+"=>Ok(self.call_rt("sail_concat",Ty::String,vec![a,b])),
                "=="|"!="=>{let v=self.call_rt("sail_str_eq",Ty::Bool,vec![a,b]);if op=="=="{Ok(v)}else{let ir=self.tmp();self.emit(format!("{ir} = xor i1 {}, true",v.ir));Ok(Value{ty:Ty::Bool,ir})}},
                _=>self.err("Strings support +, == and !="),
            }
        }
        if !matches!(a.ty,Ty::Int|Ty::Float|Ty::Bool|Ty::Char)||!matches!(b.ty,Ty::Int|Ty::Float|Ty::Bool|Ty::Char){return self.err(format!("Operator {op} requires compatible scalar operands"));}
        let ty=if a.ty==Ty::Float||b.ty==Ty::Float{Ty::Float}else{Ty::Int};
        let a=self.cast(a,&ty)?;let b=self.cast(b,&ty)?;
        let is_float=ty==Ty::Float;
        let compare=match op{"=="=>Some(("eq","oeq")),"!="=>Some(("ne","une")),"<"=>Some(("slt","olt")),"<="=>Some(("sle","ole")),">"=>Some(("sgt","ogt")),">="=>Some(("sge","oge")),_=>None};
        let ir=self.tmp();
        if let Some((int,float))=compare{
            self.emit(format!("{ir} = {} {} {} {}, {}",if is_float{"fcmp"}else{"icmp"},if is_float{float}else{int},ty.llvm(),a.ir,b.ir));
            return Ok(Value{ty:Ty::Bool,ir});
        }
        if !is_float && (op=="/"||op=="%"){return Ok(self.call_rt(if op=="/"{"sail_idiv"}else{"sail_imod"},Ty::Int,vec![a,b]));}
        let instruction=match (op,is_float){
            ("+",false)=>"add",("-",false)=>"sub",("*",false)=>"mul",
            ("+",true)=>"fadd",("-",true)=>"fsub",("*",true)=>"fmul",("/",true)=>"fdiv",("%",true)=>"frem",
            _=>return self.err(format!("Unsupported operator '{op}'")),
        };
        self.emit(format!("{ir} = {instruction} {} {}, {}",ty.llvm(),a.ir,b.ir));Ok(Value{ty,ir})
    }
    fn body(&mut self,body:&[LocatedStmt],scope:bool)->Result<(),Error>{
        if scope{self.vars.push(HashMap::new());}
        for s in body{if self.terminated{break;}self.line=s.line;self.statement(&s.stmt)?;}
        if scope{self.vars.pop();}Ok(())
    }
    fn declare(&mut self,d:&Decl,reuse:bool)->Result<(),Error>{
        let ty=self.resolve_ty(&d.ty);
        let slot=if reuse{
            if let Some(existing)=self.lookup(&d.name){
                self.vars.last_mut().unwrap().insert(d.name.clone(),existing.clone());
                existing
            }else{self.alloc_slot(&d.name,ty.clone(),false)?}
        }else{self.alloc_slot(&d.name,ty.clone(),false)?};
        if slot.ty!=ty{return self.err("Iterator type differs from the existing variable");}
        self.initialize(d,&slot)
    }
    fn initialize(&mut self,d:&Decl,slot:&Slot)->Result<(),Error>{
        let ty=slot.ty.clone();
        let value=if let Some((path,format,mode))=&d.file{
            if format!="txt"{return self.err("This release supports txt files; bin/hex encoding is not implemented");}
            let p=Value{ty:Ty::String,ir:self.gen.string(path)};let m=Value{ty:Ty::String,ir:self.gen.string(mode)};
            self.call_rt("sail_file_new",Ty::File,vec![p,m])
        }else if !d.shape.is_empty(){self.new_tensor(d,&ty)?}
        else if let Some(size)=&d.size{
            let n=self.expr(size)?;let n=self.cast(n,&Ty::Int)?;
            self.new_vector(&ty,n)?
        }else if let Some(e)=&d.init{self.expr(e)?}else{self.zero(&ty)?};
        self.store(&slot,value)?;
        if !d.values.is_empty(){
            for (i,e) in d.values.iter().enumerate(){
                let element=Expr::Index(Box::new(Expr::Var(d.name.clone())),Box::new(Expr::Int(i as i64)));
                let dest=self.slot(&element)?;let v=self.expr(e)?;self.store(&dest,v)?;
            }
        }
        Ok(())
    }
    fn statement(&mut self,s:&Stmt)->Result<(),Error>{
        match s{
            Stmt::Decl(ds)=>{for d in ds{self.declare(d,false)?;}},
            Stmt::Flow(stages)=>self.flow(stages)?,
            Stmt::Return(e)=>{
                if self.ret==Ty::Void {if e.is_some(){return self.err("void function cannot return a value");}self.emit("ret void");}
                else{let ty=self.ret.clone();let value=if let Some(e)=e{self.expr(e)?}else{self.zero(&ty)?};let v=self.cast(value,&ty)?;self.check_return_tensor(v.clone());self.emit(format!("ret {} {}",ty.llvm(),v.ir));}
                self.terminated=true;
            },
            Stmt::When(cond,yes,no)|Stmt::Infer(cond,yes,no)=>{
                let c=self.expr(cond)?;
                if matches!(s,Stmt::Infer(..)) && c.ty!=Ty::Bool{
                    return self.err("infer| requires a bool condition; use confidence ~ threshold");
                }
                let c=self.cast(c,&Ty::Bool)?;
                let yl=self.label("when.yes");let nl=self.label("when.other");let end=self.label("when.end");
                self.emit(format!("br i1 {}, label %{yl}, label %{nl}",c.ir));
                self.mark(&yl);self.body(yes,true)?;let yes_done=self.terminated;self.branch(&end);
                self.mark(&nl);self.body(no,true)?;let no_done=self.terminated;self.branch(&end);self.mark(&end);
                if yes_done&&no_done{self.emit("unreachable");self.terminated=true;}
            },
            Stmt::Escalate(state,payload)=>{
                let state=self.expr(state)?;let payload=self.expr(payload)?;
                if state.ty!=Ty::String||payload.ty!=Ty::String{
                    return self.err("^> requires a string state and a string payload");
                }
                self.call_rt("sail_escalate",Ty::Void,vec![state,payload]);
                self.emit("unreachable");self.terminated=true;
            },
            Stmt::Repeat(cond,body)=>{
                let check=self.label("repeat.check");let run=self.label("repeat.body");let end=self.label("repeat.end");
                self.branch(&check);self.mark(&check);
                let c=self.expr(cond)?;let c=self.cast(c,&Ty::Bool)?;self.emit(format!("br i1 {}, label %{run}, label %{end}",c.ir));
                self.mark(&run);self.loops.push((end.clone(),check.clone()));self.body(body,true)?;self.loops.pop();self.branch(&check);self.mark(&end);
            },
            Stmt::Iterate(decl,op,bound,step,body)=>{
                self.vars.push(HashMap::new());
                self.declare(decl,true)?;
                let check=self.label("iterate.check");let run=self.label("iterate.body");let inc=self.label("iterate.step");let end=self.label("iterate.end");
                self.branch(&check);self.mark(&check);
                let c=Expr::Binary(op.clone(),Box::new(Expr::Var(decl.name.clone())),Box::new(bound.clone()));
                let c=self.expr(&c)?;self.emit(format!("br i1 {}, label %{run}, label %{end}",c.ir));
                self.mark(&run);self.loops.push((end.clone(),inc.clone()));self.body(body,true)?;self.loops.pop();self.branch(&inc);
                self.mark(&inc);self.expr(&Expr::PostInc(Box::new(Expr::Var(decl.name.clone())),*step))?;self.branch(&check);self.mark(&end);self.vars.pop();
            },
            Stmt::Break|Stmt::Continue=>{
                let (end,next)=self.loops.last().cloned().ok_or_else(||Error::new(self.line,1,"break/continue outside a loop"))?;
                self.branch(if matches!(s,Stmt::Break){&end}else{&next});
            },
            Stmt::Action(e)=>{
                if let Expr::Call(..)=e{self.expr(e)?;return Ok(());}
                if let Expr::Member(base,n)=e{
                    if n=="open"||n=="close"{
                        let f=self.expr(base)?;if f.ty!=Ty::File{return self.err("open/close require a file");}
                        self.call_rt(if n=="open"{"sail_file_open"}else{"sail_file_close"},Ty::Void,vec![f]);return Ok(());
                    }
                    if n=="concurr"{return self.err("Multitex concurrency is not implemented");}
                }
                if let Expr::Var(n)=e {
                    if self.gen.signatures.contains_key(n){self.invoke(e,vec![],None)?;return Ok(());}
                }
                if matches!(e,Expr::PostInc(_,_)){self.expr(e)?;}else{return self.err("Bare expressions have no effect; use value > target");}
            }
        }
        Ok(())
    }
    fn print(&mut self,v:Value)->Result<(),Error>{
        let rt=match v.ty{Ty::Int=>"sail_print_int",Ty::Float=>"sail_print_float",Ty::Char=>"sail_print_char",Ty::String=>"sail_print_string",Ty::Bool=>"sail_print_bool",_=>return self.err("out only accepts scalar values")};
        self.call_rt(rt,Ty::Void,vec![v]);Ok(())
    }
    fn read(&mut self,ty:&Ty)->Result<Value,Error>{
        let rt=match ty{Ty::Int=>"sail_read_int",Ty::Float=>"sail_read_float",Ty::Char=>"sail_read_char",Ty::String=>"sail_read_string",Ty::Bool=>"sail_read_bool",_=>return self.err("in only accepts scalar destinations")};
        Ok(self.call_rt(rt,ty.clone(),vec![]))
    }
    fn function_key(&mut self,e:&Expr)->Result<Option<(String,Option<Value>)>,Error>{
        match e{
            Expr::Var(n) if self.gen.signatures.contains_key(n)=>Ok(Some((n.clone(),None))),
            Expr::Member(base,n)=>{
                if let Some(key)=clair::qualified_name(e){
                    if self.gen.signatures.contains_key(&key) && !clair::root_name(base).is_some_and(|root|self.lookup(root).is_some()){
                        return Ok(Some((key,None)));
                    }
                }
                let b=self.expr(base)?;
                if let Ty::Object(c)=&b.ty{let key=format!("{c}.{n}");if self.gen.signatures.contains_key(&key){return Ok(Some((key,Some(b))));}}
                Ok(None)
            },
            _=>Ok(None),
        }
    }
    fn invoke(&mut self,e:&Expr,mut args:Vec<Value>,operator:Option<String>)->Result<Value,Error>{
        let resolved=self.function_key(e)?.ok_or_else(||Error::new(self.line,1,"Unknown function"))?;
        self.invoke_resolved(resolved,std::mem::take(&mut args),operator)
    }
    fn invoke_resolved(&mut self,(key,object):(String,Option<Value>),mut args:Vec<Value>,operator:Option<String>)->Result<Value,Error>{
        let sig=self.gen.signatures[&key].clone();let offset=if object.is_some(){1}else{0};
        if let Some(object)=object{args.insert(0,object);}
        if key=="main"{return self.err("main cannot be called as a pipeline function");}
        if args.len()!=sig.params.len(){return self.err(format!("{key} expects {} argument(s), received {}",sig.params.len()-offset,args.len()-offset));}
        let generic=sig.params.iter().any(|t|matches!(t,Ty::Generic(_)));
        let mut symbol=sig.symbol;
        let mut specialization=None;
        if generic{
            let ty=if args.iter().any(|v|v.ty==Ty::Float){Ty::Float}else if args.iter().any(|v|v.ty==Ty::String){Ty::String}else{Ty::Int};
            symbol.push_str(&format!("__{}",ty_name(&ty)));specialization=Some(ty);
            let f=self.gen.program.functions.iter().find(|f|f.name==key).unwrap();
            if f.generic_op.is_some(){
                let op=operator.ok_or_else(||Error::new(self.line,1,"&Op generic call requires an operator, e.g. a + b > Result"))?;
                if specialization==Some(Ty::String){return self.err("&Op generics currently accept numeric types only");}
                symbol.push_str(&format!("_{}",op_name(&op)));
            }
        }
        let mut casted=Vec::new();
        for (v,ty) in args.into_iter().zip(&sig.params){
            let ty=if matches!(ty,Ty::Generic(_)){specialization.clone().unwrap()}else{ty.clone()};
            casted.push(self.cast(v,&ty)?);
        }
        let ret=if matches!(sig.ret,Ty::Generic(_)){specialization.unwrap()}else{sig.ret};
        Ok(self.call_rt(&symbol,ret,casted))
    }
    fn flow(&mut self,stages:&[Vec<Expr>])->Result<(),Error>{
        let input=matches!(stages[0].as_slice(),[Expr::Var(n)] if n=="in");
        let first_file=if stages[0].len()==1 {
            if let Expr::Var(n)=&stages[0][0]{self.lookup(n).is_some_and(|s|s.ty==Ty::File)}else{false}
        }else{false};
        let mut values=Vec::new();
        let mut pending_op=None;
        let mut start=1;
        if input||first_file{
            let dest=&stages[1];
            let file=if first_file{Some(self.expr(&stages[0][0])?)}else{None};
            for e in dest {
                if let Expr::Broadcast(base)=e{
                    if file.is_some(){return self.err("File reads require a scalar string destination");}
                    let collection=self.expr(base)?;let item=self.collection_item(&collection.ty)?;
                    let v=self.read(&item)?;values.push(self.fill_collection(collection,v)?);
                    continue;
                }
                let slot=if let Expr::Binding(d)=e{
                    if matches!(d.ty,Ty::Tensor(..)|Ty::Vector(..)){return self.err("in requires scalar pipeline declarations");}
                    self.alloc_slot(&d.name,d.ty.clone(),false)?
                }else{self.slot(e)?};
                let v=if let Some(f)=&file{
                    if slot.ty!=Ty::String{return self.err("File reads currently require a string destination");}
                    self.call_rt("sail_file_read",Ty::String,vec![f.clone()])
                }else{self.read(&slot.ty)?};
                self.store(&slot,v.clone())?;values.push(v);
            }
            start=2;
        }else{
            // &Op binds the source operator, not the already-computed result.
            let next_generic=if stages.len()>1 && stages[1].len()==1 {
                if let Expr::Var(n)=&stages[1][0]{
                    self.gen.program.functions.iter().any(|f|f.name==*n && f.generic_op.is_some())
                }else{false}
            }else{false};
            if next_generic && stages[0].len()==1{
                if let Expr::Binary(op,a,b)=&stages[0][0]{
                    if !["+","-","*","/","%"].contains(&op.as_str()){return self.err("Unsupported generic operator binding");}
                    values.push(self.expr(a)?);values.push(self.expr(b)?);pending_op=Some(op.clone());
                }else{return self.err("&Op call requires a binary arithmetic expression");}
            }else{for e in &stages[0]{values.push(self.expr(e)?);}}
        }
        for stage in &stages[start..]{
            let callable=if stage.len()==1{self.function_key(&stage[0])?}else{None};
            if matches!(stage.as_slice(),[Expr::Var(n)] if n=="$return"){
                if values.len()!=1{return self.err("Return pipeline requires exactly one value");}
                if self.ret==Ty::Void{return self.err("void function cannot return a value");}
                let ret=self.ret.clone();let v=self.cast(values.remove(0),&ret)?;
                self.check_return_tensor(v.clone());
                self.emit(format!("ret {} {}",ret.llvm(),v.ir));self.terminated=true;
            }else if matches!(stage.as_slice(),[Expr::Var(n)] if n=="out"){
                for v in &values{self.print(v.clone())?;}
            }else if let Some(resolved)=callable{
                values=vec![self.invoke_resolved(resolved,values,pending_op.take())?];
            }else if stage.len()==1 && matches!(&stage[0],Expr::Var(n) if self.lookup(n).is_some_and(|s|s.ty==Ty::File)){
                let f=self.expr(&stage[0])?;
                for v in &values{
                    if v.ty!=Ty::String{return self.err("File writes currently require strings");}
                    self.call_rt("sail_file_write",Ty::Void,vec![f.clone(),v.clone()]);
                }
            }else{
                if values.len()!=stage.len(){return self.err(format!("Pipeline has {} value(s) but {} destinations",values.len(),stage.len()));}
                let mut out=Vec::new();
                for (e,v) in stage.iter().zip(&values){
                    if let Expr::Broadcast(base)=e{
                        let collection=self.expr(base)?;out.push(self.fill_collection(collection,v.clone())?);
                    }else{
                        let s=if let Expr::Binding(d)=e{
                            let value=self.cast(v.clone(),&d.ty)?;
                            self.check_tensor_shape(value,&d.shape)?;
                            self.alloc_slot(&d.name,d.ty.clone(),false)?
                        }else{self.slot(e)?};
                        if matches!(s.ty,Ty::Tensor(..)){
                            if !matches!(e,Expr::Binding(_)){
                                let value=self.cast(v.clone(),&s.ty)?;let old=self.load(&s);
                                self.call_rt("sail_tensor_same_shape",Ty::Void,vec![old,value]);
                            }
                        }
                        self.store(&s,v.clone())?;out.push(self.load(&s));
                    }
                }
                values=out;
            }
        }
        Ok(())
    }
}
const RUNTIME_DECLS:&str = r#"
declare ptr @sail_alloc(i64)
declare ptr @sail_vec_new(i64)
declare ptr @sail_vec_at(ptr, i64)
declare ptr @sail_tensor_new(i64, ptr)
declare ptr @sail_tensor_at(ptr, i64, ptr)
declare void @sail_vec_fill(ptr, i64)
declare void @sail_tensor_fill(ptr, i64)
declare void @sail_tensor_check(ptr, i64, ptr)
declare void @sail_tensor_same_shape(ptr, ptr)
declare ptr @sail_tensor_slice(ptr, i64, ptr)
declare ptr @sail_tensor_exp(ptr, i64)
declare i64 @sail_tensor_reduce(ptr, i64, i64)
declare ptr @sail_tensor_binary(ptr, ptr, i64, i64, i64)
declare ptr @sail_tensor_scalar(ptr, i64, i64, i64, i64, i64)
declare ptr @sail_tensor_matmul(ptr, ptr, i64, i64)
declare ptr @sail_tensor_outer(ptr, ptr, i64, i64)
declare void @sail_escalate(ptr, ptr) noreturn
declare i64 @sail_strlen(ptr)
declare i64 @sail_string_at(ptr, i64)
declare i64 @sail_to_char(ptr)
declare ptr @sail_concat(ptr, ptr)
declare i1 @sail_str_eq(ptr, ptr)
declare i64 @sail_idiv(i64, i64)
declare i64 @sail_imod(i64, i64)
declare void @sail_print_int(i64)
declare void @sail_print_float(double)
declare void @sail_print_bool(i1)
declare void @sail_print_char(i64)
declare void @sail_print_string(ptr)
declare i64 @sail_read_int()
declare double @sail_read_float()
declare i1 @sail_read_bool()
declare i64 @sail_read_char()
declare ptr @sail_read_string()
declare ptr @sail_file_new(ptr, ptr)
declare void @sail_file_open(ptr)
declare void @sail_file_close(ptr)
declare i1 @sail_file_eof(ptr)
declare ptr @sail_file_read(ptr)
declare void @sail_file_write(ptr, ptr)
"#;