use super::*;

pub(super) fn qualified_name(e:&Expr)->Option<String>{
    match e{
        Expr::Var(n)=>Some(n.clone()),
        Expr::Member(base,n)=>Some(format!("{}.{n}",qualified_name(base)?)),
        _=>None,
    }
}
pub(super) fn root_name(e:&Expr)->Option<&str>{
    match e{Expr::Var(n)=>Some(n),Expr::Member(base,_)=>root_name(base),_=>None}
}
fn pure_initializer(e:&Expr,earlier:&HashSet<String>)->bool{
    match e{
        Expr::Int(_)|Expr::Float(_)|Expr::Bool(_)|Expr::String(_)=>true,
        Expr::Var(n)=>earlier.contains(n),
        Expr::Unary(_,a)=>pure_initializer(a,earlier),
        Expr::Binary(_,a,b)=>pure_initializer(a,earlier)&&pure_initializer(b,earlier),
        _=>false,
    }
}
impl Generator<'_>{
    pub(super) fn prepare_module_data(&mut self)->Result<(),Error>{
        let mut earlier=HashSet::new();
        for (i,d) in self.program.declarations.iter().enumerate(){
            if earlier.contains(&d.name)||self.signatures.contains_key(&d.name){
                return Err(Error::new(1,1,format!("Duplicate module name '{}'",d.name)));
            }
            if matches!(d.ty,Ty::File|Ty::Object(_)|Ty::Generic(_)|Ty::Void){
                return Err(Error::new(1,1,"Module data supports scalar, vector, and tensor declarations only"));
            }
            if !d.init.iter().chain(d.size.iter()).chain(d.shape.iter()).chain(d.values.iter()).all(|e|pure_initializer(e,&earlier)){
                return Err(Error::new(1,1,"Module initializers require literals, earlier module data, or pure arithmetic; calls, forward references and side effects are not allowed"));
            }
            let ptr=format!("@.sail.global.{i}");
            self.globals.push(format!("{ptr} = private global {} zeroinitializer",d.ty.llvm()));
            self.module_slots.insert(d.name.clone(),Slot{ty:d.ty.clone(),ptr});
            earlier.insert(d.name.clone());
        }
        Ok(())
    }
}
impl FunctionGen<'_,'_>{
    pub(super) fn tensor_extension(&mut self,name:&str,receiver:Value,mut args:Vec<Value>)->Result<Value,Error>{
        let function=format!("Tensor.{name}");
        if !self.gen.signatures.contains_key(&function){
            return self.err(format!("Tensor.{name} is not defined. Supply an explicit Sail function; confidence and axiom verification have no implicit AI semantics"));
        }
        args.insert(0,receiver);
        // Use the ordinary call path, retaining type and runtime shape checks.
        self.invoke(&Expr::Var(function),args,None)
    }
    pub(super) fn call_expression(&mut self,target:&Expr,args:&[Expr])->Result<Value,Error>{
        if let Expr::Member(base,name)=target{
            // Defined namespaces (Tokenizer.encode, etc.) use normal Sail functions.
            let namespace=qualified_name(target).is_some_and(|key|self.gen.signatures.contains_key(&key))
                && !root_name(base).is_some_and(|root|self.lookup(root).is_some());
            if !namespace{
                // Resolve writable append receivers once (e.g. words|i++|).
                let append_slot=if name=="append"{Some(self.slot(base)?)}else{None};
                let receiver=if let Some(slot)=&append_slot{self.load(slot)}else{self.expr(base)?};
                if matches!(receiver.ty,Ty::Tensor(..)){
                    let mut values=Vec::new();for e in args{values.push(self.expr(e)?);}
                    if name=="cross"{
                        if values.len()!=1{return self.err("cross expects exactly one rank-1 tensor argument");}
                        return self.tensor_outer(receiver,values.remove(0));
                    }
                    if args.is_empty() && ["sum","max","argmax","exp"].contains(&name.as_str()){
                        return self.tensor_member(receiver,name);
                    }
                    return self.tensor_extension(name,receiver,values);
                }
                if receiver.ty==Ty::String && name=="append"{
                    if args.len()!=1{return self.err("string.append expects exactly one string argument");}
                    let value=self.expr(&args[0])?;
                    if value.ty!=Ty::String{return self.err("string.append requires a string argument");}
                    let slot=append_slot.unwrap();
                    let result=self.call_rt("sail_concat",Ty::String,vec![receiver,value]);
                    self.store(&slot,result.clone())?;
                    return Ok(result);
                }
                if let Ty::Object(class)=&receiver.ty{
                    let key=format!("{class}.{name}");
                    if self.gen.signatures.contains_key(&key){
                        let mut values=Vec::new();for e in args{values.push(self.expr(e)?);}
                        return self.invoke_resolved((key,Some(receiver)),values,None);
                    }
                }
                return self.err(format!("Unknown callable member '{name}'"));
            }
        }
        let mut values=Vec::new();for e in args{values.push(self.expr(e)?);}
        self.invoke(target,values,None)
    }
    fn tensor_outer(&mut self,a:Value,b:Value)->Result<Value,Error>{
        let ak=match &a.ty{Ty::Tensor(t,1)=>self.numeric_kind(t)?,_=>return self.err("cross requires two numeric rank-1 tensors (outer product)")};
        let bk=match &b.ty{Ty::Tensor(t,1)=>self.numeric_kind(t)?,_=>return self.err("cross requires two numeric rank-1 tensors (outer product)")};
        let item=if ak==1||bk==1{Ty::Float}else{Ty::Int};
        Ok(self.call_rt("sail_tensor_outer",Ty::Tensor(Box::new(item),2),vec![a,b,Self::int(ak),Self::int(bk)]))
    }
}
