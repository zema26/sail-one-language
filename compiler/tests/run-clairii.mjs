// Real compiler/runtime checks, not browser automation. Node 24.
import assert from 'node:assert/strict';
import {execFileSync,spawnSync} from 'node:child_process';
import {mkdtempSync,readFileSync,writeFileSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {executeSail} from '../../artifacts/sail-playground/src/lib/wasm-runtime.ts';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..');
const compiler=path.join(root,'compiler/target/release/sailc');
const dir=mkdtempSync(path.join(tmpdir(),'sail-clairii-'));
const wrap=(body,top='')=>`${top}\nfun| > main > int\n${body}\n0 > |back|\n|fun\n`;
const cases=[
 ['outer product row-major and scalar scaling',wrap("int a|1, 2| b|1, 3| r|2, 2, 3|\n2 > a|0|\n-3 > a|1|\n4 > b|0|\n5 > b|1|\n6 > b|2|\na.cross(b) > r\nr|0, 2| ' ' r|1, 0| ' ' r.sum > out"),'12 -12 -15'],
 ['mixed outer product and independent storage',wrap("int a|1, 2|\n2 > a|*|\nfloat b|1, 3| r|2, 2, 3|\n0.5 > b|*|\na.cross(b) * 0.01 > r\n99 > a|0|\nr|0, 0| ' ' r.sum > out"),'0.01 0.06'],
 ['empty outer product',wrap("int a|1, 0| b|1, 3| r|2, 0, 3|\na.cross(b) > r\nr.length ' ' r.sum > out"),'0 0'],
 ['int64 outer product wraps',wrap("int a|1, 1| b|1, 1| r|2, 1, 1|\n9223372036854775807 > a|0|\n2 > b|0|\na.cross(b) > r\nr|0, 0| > out"),'-2'],
 ['shared module rebinding and scoped shadowing',wrap("Update()\nfloat memory|2, 2, 2|\n7.0 > memory|*|\nLocal()\nRead() ' ' counter ' ' memory.sum > out","float memory|2, 2, 2|\nint counter(0)\nfun| > Update > void\nmemory + 1.0 > memory\ncounter++\n|fun\nfun| > Local > void\nint counter(99)\ncounter++\n|fun\nfun| > Read > float\nmemory.sum > |back|\n|fun"),'4 1 28'],
 ['module literals vectors and shape expressions',wrap("flag ' ' text ' ' values|1| ' ' matrix.length > out","int n(2)\nbool flag(true)\nstring text('ok')\nint values|n|(|3, 7|)\nfloat matrix|2, n, n + 1|"),'true ok 7 6'],
 ['inline declarations calls and independent strings',wrap("string s('A')\ns > string snapshot\ns.append('B')\ns.append(s)\nAdd(2, Add(3, 4)) > int n\nn > float f\ntrue > bool valid\nf ' ' valid ' ' snapshot ' ' s > out","fun| int a int b > Add > int\na + b > |back|\n|fun"),'9 true A ABAB'],
 ['append target index evaluated once',wrap("string words|2|\nint i(0)\nwords|i++|.append('x')\ni ' ' words|0| ':' words|1| > out"),'1 x:'],
 ['typed tensor destination and indexing call result',wrap("Make() > float a|1, 2|\nMake()|1| ' ' a.sum > out","fun| > Make > float|1, 2|\nfloat a|1, 2|\n3.0 > a|*|\na > |back|\n|fun"),'3 6'],
 ['custom tensor policy and namespace symbol distinction',wrap("float a|1, 2|\n2.0 > a|*|\na.confidence ' ' a.confidence() ' ' A.b(2) ' ' A_b(2) > out","fun| float a|1, 2| > Tensor.confidence > float\na.sum > |back|\n|fun\nfun| int n > A.b > int\nn + 1 > |back|\n|fun\nfun| int n > A_b > int\nn + 2 > |back|\n|fun"),'4 4 3 4'],
 ['member calls evaluate receiver once',wrap("Make().Value(3) ' ' count > out","int count(0)\nclass| Box\nfun| int n > Value > int\nn > |back|\n|fun\n|class\nfun| > Make > Box\nBox b\ncount++\nb > |back|\n|fun"),'3 1'],
 ['alternate nested other ending then following statement',wrap("when| true\nwhen| false\n'bad' > out\nother|\n'ok' > out\n|other\nother|\n'bad' > out\n|other\n'!' > out"),'ok!'],
];
const failures=[
 ['global allocation checked before main',wrap("'bad' > out","float huge|2, 8192, 8192|"),/limit/i],
 ['outer product allocation budget',wrap("float a|1, 1000| b|1, 1000|\na.cross(b) > float r|2, 1000, 1000|"),/limit/i],
 ['typed tensor destination checks shape',wrap("float a|1, 2|\na > float b|1, 3|"),/shape mismatch/i],
 ['custom tensor policy checks receiver shape',wrap("float a|1, 3|\na.confidence > out","fun| float a|1, 2| > Tensor.confidence > float\na.sum > |back|\n|fun"),/shape mismatch/i],
 ['module rebinding preserves shape',wrap("float a|1, 3|\na > shared","float shared|1, 2|"),/shape mismatch/i],
];
function build(source,target,name,opt){
 const file=path.join(dir,`${name}.sl`),out=path.join(dir,`${name}.${target}`);
 writeFileSync(file,source);execFileSync(compiler,[file,`--${target}`,`-O${opt}`,'-o',out],{timeout:15000,stdio:'pipe'});
 return out;
}
try{
 for(const opt of [0,2]){
  for(const [i,[name,source,expected]] of cases.entries()){
   const wasm=build(source,'wasm',`ok-${i}`,opt);
   const result=await executeSail(new Uint8Array(readFileSync(wasm)),'',()=>{});
   assert.equal(result.exitCode,0);assert.equal(result.stdout,expected,`${name} Wasm`);
   const native=spawnSync(build(source,'native',`ok-${i}`,opt),{encoding:'utf8',timeout:3000,cwd:dir});
   assert.equal(native.status,0,`${name}: ${native.stderr}`);assert.equal(native.stdout,expected,`${name} native`);
   console.info(`PASS ${name}: Wasm + native -O${opt}`);
  }
  for(const [i,[name,source,pattern]] of failures.entries()){
   const wasm=readFileSync(build(source,'wasm',`fail-${i}`,opt));
   await assert.rejects(()=>executeSail(new Uint8Array(wasm),'',()=>{}),pattern,name);
   const native=spawnSync(build(source,'native',`fail-${i}`,opt),{encoding:'utf8',timeout:3000});
   assert.equal(native.status,70,`${name}: ${native.stderr}`);assert.match(native.stderr,pattern);
   console.info(`PASS ${name}: explicit failure on both targets -O${opt}`);
  }
 }
 const source=readFileSync(path.join(root,'compiler/examples/clairii.sl'),'utf8');
 const wasm=readFileSync(build(source,'wasm','clairii',2));
 const native=build(source,'native','clairii',2);
 for(const input of ['hello','veto','uncertain']){
  const result=await executeSail(new Uint8Array(wasm),input,()=>{});
  const process=spawnSync(native,{input,encoding:'utf8',timeout:3000,cwd:dir});
  assert.equal(process.status,result.exitCode);assert.equal(process.stdout,result.stdout);
  if(input==='hello'){
   assert.equal(result.exitCode,0);assert.equal(result.stdout,'accepted <EOT>\nUpdates: 2\nLiquid memory[0,0]: 0.0201\n');
   // A fresh module run must not inherit previous liquid memory.
   const again=await executeSail(new Uint8Array(wasm),input,()=>{});
   assert.equal(again.stdout,result.stdout);
  }else{
   assert.equal(result.exitCode,75);assert.deepEqual(result.escalation,{status:'HALT_AND_ESCALATE',state:'TEMPORAL_LOCK',payload:''});
   assert.deepEqual(JSON.parse(process.stderr),result.escalation);
   assert.match(result.stdout,input==='veto'?/^VETO:/:/^HALT_AND_ESCALATE:/);
  }
  console.info(`PASS ClairII ${input}: Wasm + native`);
 }
 // CLI linked module data must initialize in dependency order.
 writeFileSync(path.join(dir,'data.sl'),"module| Data\nint width(2)\nfloat liquid|2, width, width|\nfun| > Update > void\nliquid + 0.5 > liquid\n|fun\n|module\n");
 const linked=path.join(dir,'linked.sl');
 writeFileSync(linked,"module| App\nlink| 'data.sl' |link\nint count(width + 1)\nfun| > main > int\nUpdate()\nliquid.sum ' ' count > out\n0 > |back|\n|fun\n|module\n");
 for(const target of ['wasm','native']){
  const out=path.join(dir,`linked.${target}`);
  execFileSync(compiler,[linked,`--${target}`,'-o',out],{timeout:15000});
  const stdout=target==='wasm'?(await executeSail(new Uint8Array(readFileSync(out)),'',()=>{})).stdout:execFileSync(out,{encoding:'utf8',timeout:3000});
  assert.equal(stdout,'2 3');
 }
 console.info('PASS CLI-linked module initialization: Wasm + native');
}finally{rmSync(dir,{recursive:true,force:true});}
