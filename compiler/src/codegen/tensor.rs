use super::*;

impl FunctionGen<'_, '_> {
    pub(super) fn check_return_tensor(&mut self,value:Value){
        if let Some((dims,rank))=self.ret_contract.clone(){
            self.call_rt("sail_tensor_check",Ty::Void,vec![value,Self::int(rank as i64),dims]);
        }
    }
    pub(super) fn check_tensor_shape(&mut self, value:Value, shape:&[Expr])->Result<(),Error>{
        if shape.is_empty(){return Ok(());}
        let dims=self.int_array(shape)?;
        self.call_rt("sail_tensor_check",Ty::Void,vec![value,Self::int(shape.len() as i64),dims]);
        Ok(())
    }
    pub(super) fn tensor_read(&mut self,base:Value,indices:&[Expr])->Result<Value,Error>{
        let (item,rank)=match &base.ty{Ty::Tensor(t,r)=>(*t.clone(),*r),_=>return self.err("Multiple coordinates require a tensor")};
        if indices.len()==rank{let slot=self.tensor_slot(base,indices)?;return Ok(self.load(&slot));}
        if indices.is_empty()||indices.len()>rank{return self.err("Too many tensor coordinates");}
        let coords=self.int_array(indices)?;
        Ok(self.call_rt("sail_tensor_slice",Ty::Tensor(Box::new(item),rank-indices.len()),vec![base,Self::int(indices.len() as i64),coords]))
    }
    pub(super) fn numeric_kind(&self,ty:&Ty)->Result<i64,Error>{
        match ty{Ty::Int=>Ok(0),Ty::Float=>Ok(1),_=>self.err("Tensor math requires int or float elements and scalar operands")}
    }
    pub(super) fn tensor_member(&mut self,base:Value,name:&str)->Result<Value,Error>{
        let (item,rank)=match &base.ty{Ty::Tensor(t,r)=>(*t.clone(),*r),_=>unreachable!()};
        let kind=Self::int(self.numeric_kind(&item)?);
        if name=="exp"{
            return Ok(self.call_rt("sail_tensor_exp",Ty::Tensor(Box::new(Ty::Float),rank),vec![base,kind]));
        }
        let mode=match name{"sum"=>0,"max"=>1,"argmax"=>2,_=>unreachable!()};
        let bits=self.call_rt("sail_tensor_reduce",Ty::Int,vec![base,kind,Self::int(mode)]);
        if name=="argmax"||item==Ty::Int{return Ok(bits);}
        let ir=self.tmp();self.emit(format!("{ir} = bitcast i64 {} to double",bits.ir));
        Ok(Value{ty:Ty::Float,ir})
    }
    pub(super) fn tensor_binary(&mut self,op:&str,a:Value,b:Value)->Result<Value,Error>{
        let operation=match op{"+"=>0,"-"=>1,"*"=>2,"/"=>3,_=>return self.err("Tensor arithmetic supports +, -, *, / only")};
        let (at,ar)=match &a.ty{Ty::Tensor(t,r)=>(*t.clone(),Some(*r)),t=>(t.clone(),None)};
        let (bt,br)=match &b.ty{Ty::Tensor(t,r)=>(*t.clone(),Some(*r)),t=>(t.clone(),None)};
        let ak=self.numeric_kind(&at)?;let bk=self.numeric_kind(&bt)?;
        let item=if ak==1||bk==1{Ty::Float}else{Ty::Int};
        match (ar,br){
            (Some(ar),Some(br))=>{
                if op=="*"{
                    if ar>2||br>2{return self.err("Tensor multiplication supports rank-1 vectors and rank-2 matrices only");}
                    let rank=if ar==1&&br==1{1}else if ar==2&&br==2{2}else{1};
                    let v=self.call_rt("sail_tensor_matmul",Ty::Tensor(Box::new(item),rank),vec![a,b,Self::int(ak),Self::int(bk)]);
                    if ar==1&&br==1{return self.tensor_read(v,&[Expr::Int(0)]);}
                    return Ok(v);
                }
                if ar!=br{return self.err("Element-wise tensor arithmetic requires equal ranks");}
                Ok(self.call_rt("sail_tensor_binary",Ty::Tensor(Box::new(item),ar),vec![a,b,Self::int(operation),Self::int(ak),Self::int(bk)]))
            },
            _=>{
                let (tensor,scalar,rank,tk,sk,reverse)=if let Some(rank)=ar{(a,b,rank,ak,bk,0)}else{(b,a,br.unwrap(),bk,ak,1)};
                let bits=if scalar.ty==Ty::Float{
                    let ir=self.tmp();self.emit(format!("{ir} = bitcast double {} to i64",scalar.ir));Value{ty:Ty::Int,ir}
                }else{scalar};
                Ok(self.call_rt("sail_tensor_scalar",Ty::Tensor(Box::new(item),rank),vec![tensor,bits,Self::int(operation),Self::int(tk),Self::int(sk),Self::int(reverse)]))
            },
        }
    }
}
