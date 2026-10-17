// Real LLVM execution on both targets. Node 24; build sailc first.
import assert from 'node:assert/strict';
import {execFileSync, spawnSync} from 'node:child_process';
import {mkdtempSync, readFileSync, writeFileSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {executeSail} from '../../artifacts/sail-playground/src/lib/wasm-runtime.ts';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..');
const compiler=path.join(root,'compiler/target/release/sailc');
const dir=mkdtempSync(path.join(tmpdir(),'sail-math-'));
const wrap=(body,functions='')=>`module| T\n${functions}\nfun| > main > int\n${body}\n0 > |back|\n|fun\n|module\n`;
const id="fun| int n float a|1, n| > Id > float|1, n|\na > |back|\n|fun\n";
const cases=[
 ['matrix multiplication',wrap("int a|2, 2, 3| b|2, 3, 2| c|2, 2, 2|\n2 > a|*|\n3 > b|*|\na * b > c\nc|0, 0| ' ' c|1, 1| ' ' c.sum > out"),'18 18 72'],
 ['vector matrix and matrix vector',wrap("int v|1, 3| a|2, 3, 2| w|1, 2| r|1, 2| s|1, 3|\n2 > v|*|\n3 > a|*|\n4 > w|*|\nv * a > r\na * w > s\nr|1| ' ' s|2| ' ' (v * v) > out"),'18 24 12'],
 ['nonuniform row-major matrix product',wrap("int a|2, 2, 2| b|2, 2, 2| r|2, 2, 2|\n1 > a|0, 0|\n2 > a|0, 1|\n3 > a|1, 0|\n4 > a|1, 1|\n5 > b|0, 0|\n6 > b|0, 1|\n7 > b|1, 0|\n8 > b|1, 1|\na * b > r\nr|0, 0| ' ' r|0, 1| ' ' r|1, 0| ' ' r|1, 1| > out"),'19 22 43 50'],
 ['mixed scalar arithmetic and operand order',wrap("int a|1, 2|\n4 > a|*|\nfloat b|1, 2|\na + 0.5 > b\nb|0| ' ' > out\n10.0 - a > b\nb|0| ' ' > out\n12.0 / a > b\nb|0| ' ' > out\n-a > a\na|0| > out"),'4.5 6 3 -4'],
 ['elementwise mixed tensor arithmetic',wrap("int a|1, 2|\n4 > a|*|\nfloat b|1, 2| c|1, 2|\n2.0 > b|*|\na + b > c\nc|0| ' ' > out\na - b > c\nc|0| ' ' > out\na / b > c\nc|1| > out"),'6 2 2'],
 ['reductions and earliest tie index',wrap("int a|2, 2, 2|\n-5 > a|*|\n8 > a|0, 1|\n8 > a|1, 0|\na.sum ' ' a.max ' ' a.argmax > out"),'6 8 1'],
 ['integer reduction precision and wrapping',wrap("int a|1, 2|\n9007199254740993 > a|0|\n9007199254740994 > a|1|\na.max ' ' a.argmax ' ' > out\n9223372036854775807 > a|*|\na.sum > out"),'9007199254740994 1 -2'],
 ['exponential maps promote int to float',wrap("int a|1, 2|\n1 > a|1|\nfloat b|1, 2|\na.exp > b\nb|0| ' ' b|1| > out"),'1 2.71828182846'],
 ['slice copies do not mutate source',wrap("int a|3, 2, 2, 2| b|1, 2|\n5 > a|*|\na|1, 0| > b\n7 > b|0|\na|1, 0, 0| ' ' b|0| ' ' b.length > out"),'5 7 2'],
 ['empty sum and exponential',wrap("float a|1, 0| b|1, 0|\na.exp > b\nb.sum ' ' b.length > out"),'0 0'],
 ['zero-inner-dimension matrix product',wrap("float a|2, 2, 0| b|2, 0, 3| r|2, 2, 3|\na * b > r\nr.sum ' ' r.length > out"),'0 6'],
 ['dependent tensor signature',wrap("float a|1, 3| b|1, 3|\n2.5 > a|*|\n3 a > Id > b\nb.sum > out",id),'7.5'],
 ['return allocation survives function exit',wrap("float a|1, 2|\n0 > Make > a\na.sum > out","fun| int unused > Make > float|1, 2|\nfloat a|1, 2|\n4.0 > a|*|\na > |back|\n|fun\n"),'8'],
 ['parameter mutation and local rebinding',wrap("float a|1, 2|\n0.0 > a|*|\na > Update\na.sum > out","fun| float a|1, 2| > Update > int\n3.0 > a|0|\nfloat other|1, 2|\n9.0 > other|*|\nother > a\n0 > |back|\n|fun\n"),'3'],
 ['matrix mixed dtype product',wrap("int a|2, 1, 2|\n3 > a|*|\nfloat b|2, 2, 1| r|2, 1, 1|\n0.5 > b|*|\na * b > r\nr|0, 0| > out"),'3'],
 ['integer division and scalar broadcasting',wrap("int a|1, 2| b|1, 2|\n9 > a|*|\na / 2 > b\nb|0| ' ' > out\n18 / a > b\nb|1| > out"),'4 2'],
 ['dependent return contract captured at entry',wrap("float a|1, 2| b|1, 2|\n2 a > Preserve > b\nb.length > out","fun| int n float a|1, n| > Preserve > float|1, n|\n3 > n\na > |back|\n|fun\n"),'2'],
];
const failures=[
 ['parameter shape mismatch',wrap("float a|1, 2| b|1, 2|\n3 a > Id > b",id),/shape mismatch/i],
 ['return shape mismatch',wrap("float a|1, 2|\n0 > Wrong > a","fun| int n > Wrong > float|1, 2|\nfloat b|1, 3|\nb > |back|\n|fun\n"),/shape mismatch/i],
 ['assignment shape mismatch',wrap("float a|1, 2| b|1, 3|\nb > a"),/shape mismatch/i],
 ['elementwise shape mismatch',wrap("float a|2, 2, 3| b|2, 3, 2|\na + b > a"),/shape mismatch/i],
 ['product inner dimension mismatch',wrap("float a|2, 2, 3| b|2, 2, 3|\na * b > a"),/shape mismatch/i],
 ['empty maximum',wrap("float a|1, 0|\na.max > out"),/empty tensor reduction/i],
 ['empty argmax',wrap("int a|1, 0|\na.argmax > out"),/empty tensor reduction/i],
 ['NaN maximum',wrap("float a|1, 2|\n(0.0 / 0.0) > a|1|\na.max > out"),/NaN in tensor maximum/i],
 ['NaN argmax',wrap("float a|1, 2|\n(0.0 / 0.0) > a|0|\na.argmax > out"),/NaN in tensor maximum/i],
 ['integer tensor divide by zero',wrap("int a|1, 2|\na / 0 > a"),/division by zero/i],
 ['slice bounds',wrap("int a|2, 2, 2| b|1, 2|\na|2| > b"),/bounds/i],
 ['product work budget',wrap("float a|2, 400, 400| b|2, 400, 400|\na * b > a"),/operation budget/i],
];
function build(source,target,name,opt){
 const input=path.join(dir,`${name}.sl`),output=path.join(dir,`${name}.${target}`);
 writeFileSync(input,source);
 execFileSync(compiler,[input,`--${target}`,`-O${opt}`,'-o',output],{timeout:15000,stdio:'pipe'});
 return output;
}
try{
 for(const opt of [0,2]){
  for(const [i,[name,source,expected]] of cases.entries()){
   const wasm=build(source,'wasm',`case-${i}`,opt);
   const result=await executeSail(new Uint8Array(readFileSync(wasm)),'',()=>{});
   assert.equal(result.stdout,expected,`${name}: Wasm`);assert.equal(result.exitCode,0);
   const native=spawnSync(build(source,'native',`case-${i}`,opt),{encoding:'utf8',timeout:3000});
   assert.equal(native.status,0,`${name}: native ${native.stderr}`);assert.equal(native.stdout,expected,`${name}: native`);
   console.log(`PASS ${name}: Wasm + native -O${opt}`);
  }
  for(const [i,[name,source,message]] of failures.entries()){
   const wasm=build(source,'wasm',`error-${i}`,opt);
   await assert.rejects(()=>executeSail(new Uint8Array(readFileSync(wasm)),'',()=>{}),message,name);
   const native=spawnSync(build(source,'native',`error-${i}`,opt),{encoding:'utf8',timeout:3000});
   assert.equal(native.status,70,`${name}: ${native.stderr}`);assert.match(native.stderr,message);
   console.log(`PASS ${name}: explicit failure on both targets -O${opt}`);
  }
 }
 const example=readFileSync(path.join(root,'compiler/examples/tensor-math.sl'),'utf8');
 const wasm=build(example,'wasm','decoder',2),native=build(example,'native','decoder',2);
 for(const input of ['1','0']){
  const result=await executeSail(new Uint8Array(readFileSync(wasm)),input,()=>{});
  const processResult=spawnSync(native,{input,encoding:'utf8',timeout:3000});
  if(input==='1'){assert.equal(result.stdout,'Output: candidate\n');assert.equal(result.exitCode,0);assert.equal(processResult.stdout,result.stdout);assert.equal(processResult.status,0);}
  else{assert.equal(result.exitCode,75);assert.deepEqual(result.escalation,{status:'HALT_AND_ESCALATE',state:'TEMPORAL_LOCK',payload:'other'});assert.equal(processResult.status,75);assert.deepEqual(JSON.parse(processResult.stderr),result.escalation);}
  console.log(`PASS full tensor decoder input ${input}: Wasm + native`);
 }
}finally{rmSync(dir,{recursive:true,force:true});}
