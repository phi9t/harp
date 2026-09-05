const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { discoverChapters, readSource, candidateHash, chapterModule } = require('./inputs.cjs');
function fixture(t) {
  const root=fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(),'harp-pdf-inputs-')));
  t.after(()=>fs.rmSync(root,{recursive:true,force:true}));
  const dir=path.join(root,'knowledge/crouzeix_textbook/part_01_fixture');fs.mkdirSync(dir,{recursive:true});
  for(let i=1;i<=36;i++)fs.writeFileSync(path.join(dir,`${String(i).padStart(2,'0')}_chapter.md`),'# Chapter');
  return {root,dir};
}
test('filesystem discovery accepts a complete candidate without Git history',t=>{
  const {root,dir}=fixture(t); assert.equal(discoverChapters(root).length,36);
  fs.unlinkSync(path.join(dir,'36_chapter.md'));assert.throws(()=>discoverChapters(root),/36|Missing/);
});
test('duplicate chapter numbers cannot replace a required chapter',t=>{
  const {root,dir}=fixture(t);fs.renameSync(path.join(dir,'13_chapter.md'),path.join(dir,'12_duplicate.md'));
  assert.throws(()=>discoverChapters(root),/sequence/);
});
test('source reads reject symlinks and repository escapes',t=>{
  const {root,dir}=fixture(t);fs.symlinkSync('/etc/passwd',path.join(dir,'linked.md'));
  assert.throws(()=>readSource(root,'knowledge/crouzeix_textbook/part_01_fixture/linked.md'),/Symlink/);
  assert.throws(()=>readSource(root,'../outside'),/escapes/);
  fs.symlinkSync(dir,path.join(root,'alias'));
  assert.throws(()=>readSource(root,'alias/01_chapter.md'),/Symlink/);
});
test('candidate hash is order-independent and changes with source bytes',()=>{
  const a={path:'a',sha256:'1'},b={path:'b',sha256:'2'};
  assert.equal(candidateHash([a,b]),candidateHash([b,a]));
  assert.notEqual(candidateHash([a]),candidateHash([{...a,sha256:'3'}]));
});

test('Lean chapter module discovery uses the actual module directory and rejects ambiguity',t=>{
  const {root}=fixture(t);
  const base=path.join(root,'formalization/lean/CrouzeixTextbook');
  fs.mkdirSync(path.join(base,'Part06'),{recursive:true});
  fs.writeFileSync(path.join(base,'Part06/Chapter29.lean'),'namespace Example');
  assert.equal(chapterModule(root,29),'formalization/lean/CrouzeixTextbook/Part06/Chapter29.lean');
  assert.throws(()=>chapterModule(root,1),/Expected exactly one Lean module/);
  fs.mkdirSync(path.join(base,'Part05'));
  fs.writeFileSync(path.join(base,'Part05/Chapter29.lean'),'namespace Duplicate');
  assert.throws(()=>chapterModule(root,29),/Expected exactly one Lean module/);
});

test('Lean chapter module discovery rejects symlinked module files',t=>{
  const {root,dir}=fixture(t);
  const part=path.join(root,'formalization/lean/CrouzeixTextbook/Part01');
  fs.mkdirSync(part,{recursive:true});
  fs.symlinkSync(path.join(dir,'01_chapter.md'),path.join(part,'Chapter01.lean'));
  assert.throws(()=>chapterModule(root,1),/Symlink/);
});

test('Chapter 36 has a discoverable module and Chapter 37 is rejected',t=>{
  const {root}=fixture(t);
  const part=path.join(root,'formalization/lean/CrouzeixTextbook/Part06');
  fs.mkdirSync(part,{recursive:true});
  fs.writeFileSync(path.join(part,'Chapter36.lean'),'namespace Example');
  assert.equal(chapterModule(root,36),'formalization/lean/CrouzeixTextbook/Part06/Chapter36.lean');
  assert.throws(()=>chapterModule(root,37),/Invalid chapter number/);
});
