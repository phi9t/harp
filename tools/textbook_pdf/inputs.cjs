'use strict';
const fs=require('node:fs');
const path=require('node:path');
const crypto=require('node:crypto');
const bookRoot='knowledge/crouzeix_textbook';
function sourcePath(root,relative) {
  root=fs.realpathSync(root);
  const target=path.resolve(root,relative);
  if(!target.startsWith(root+path.sep))throw new Error(`Source escapes repository: ${relative}`);
  let cursor=root;
  for(const component of path.relative(root,target).split(path.sep)) {
    cursor=path.join(cursor,component);
    if(fs.lstatSync(cursor).isSymbolicLink())throw new Error(`Symlinked source: ${relative}`);
  }
  return target;
}
function readSource(root,relative) {
  const target=sourcePath(root,relative);
  if(!fs.statSync(target).isFile())throw new Error(`Source is not a file: ${relative}`);
  return fs.readFileSync(target);
}
function discoverChapters(root) {
  const chapters=[];
  const base=sourcePath(root,bookRoot);
  for(const part of fs.readdirSync(base).filter(p=>/^part_\d+/.test(p))) {
    const directory=sourcePath(root,`${bookRoot}/${part}`);
    if(!fs.statSync(directory).isDirectory())throw new Error(`Invalid chapter directory: ${part}`);
    for(const name of fs.readdirSync(directory).filter(p=>/^\d\d_.*\.md$/.test(p))) {
      const relative=`${bookRoot}/${part}/${name}`;readSource(root,relative);chapters.push(relative);
    }
  }
  chapters.sort((a,b)=>Number(path.basename(a).slice(0,2))-Number(path.basename(b).slice(0,2)));
  if(chapters.length!==35)throw new Error(`Expected 35 canonical chapter paths, found ${chapters.length}`);
  if(chapters.some((p,i)=>Number(path.basename(p).slice(0,2))!==i+1))throw new Error('Invalid chapter sequence: expected chapters 01 through 35 exactly once');
  return chapters;
}
function candidateHash(sources) {
  return crypto.createHash('sha256').update(JSON.stringify([...sources].map(({path,sha256})=>({path,sha256})).sort((a,b)=>a.path<b.path?-1:a.path>b.path?1:0))).digest('hex');
}
function chapterModule(root,chapter) {
  if(!Number.isInteger(chapter)||chapter<1||chapter>35)throw new Error(`Invalid chapter number: ${chapter}`);
  const base='formalization/lean/CrouzeixTextbook';
  const filename=`Chapter${String(chapter).padStart(2,'0')}.lean`;
  const matches=[];
  for(const part of fs.readdirSync(sourcePath(root,base)).filter(name=>/^Part\d\d$/.test(name))) {
    const directory=sourcePath(root,`${base}/${part}`);
    if(fs.readdirSync(directory).includes(filename)) {
      const relative=`${base}/${part}/${filename}`;
      readSource(root,relative);matches.push(relative);
    }
  }
  if(matches.length!==1)throw new Error(`Expected exactly one Lean module for chapter ${chapter}, found ${matches.length}`);
  return matches[0];
}
module.exports={bookRoot,sourcePath,readSource,discoverChapters,candidateHash,chapterModule};
