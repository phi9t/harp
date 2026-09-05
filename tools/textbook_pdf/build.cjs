#!/usr/bin/env node
'use strict';

const fs = require('node:fs');
const path = require('node:path');
const { execFileSync } = require('node:child_process');
const { createRequire } = require('node:module');
const { pathToFileURL } = require('node:url');
const crypto = require('node:crypto');

const { bookRoot, sourcePath, readSource, discoverChapters, candidateHash, chapterModule } = require('./inputs.cjs');
const root = path.resolve(__dirname, '../..');
const out = path.join(root, 'output/pdf');
const requireAtlas = createRequire(path.join(root, 'atlas/package.json'));
function dependency(name) {
  try { return require(name); } catch (error) {
    if (error.code !== 'MODULE_NOT_FOUND') throw error;
    return requireAtlas(name);
  }
}
const { marked } = dependency('marked');
const katex = dependency('katex');
const { chromium } = dependency('playwright');
const { PDFDocument, PDFName, PDFString, PDFHexString, PDFArray, PDFDict, StandardFonts, rgb, beginMarkedContent, endMarkedContent } = dependency('pdf-lib');
const escape = s => String(s).replace(/[&<>"']/g, c => ({ '&':'&amp;', '<':'&lt;', '>':'&gt;', '"':'&quot;', "'":'&#39;' }[c]));
const slug = s => s.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
const git = (...args) => execFileSync('git', args, { cwd: root, encoding: 'utf8' }).trim();
let commit, branch, date;
let release = true;
let manifest = { math_count: 0, sources: [] };
let chapters, glossary, sources;
const parts = ['Linear structure', 'Geometry and calculus', 'Analysis and complex functions', 'Finite-dimensional operator theory', 'Crouzeix machinery', 'Constant-two routes'];
function initialize() {
  for(const [name,expected] of Object.entries(require('./package.json').dependencies)) {
    const actual=dependency(name+'/package.json').version;
    if(actual!==expected)throw new Error(`Dependency ${name}: expected ${expected}, found ${actual}`);
  }
  commit=git('rev-parse','HEAD');branch=git('branch','--show-current');date=new Date().toISOString().slice(0,10);
  manifest={schema_version:2,edition:'working-draft-reading-edition',date,commit,branch,source_mode:'filesystem-candidate',release_validation:release,generator:{node:process.version,packages:Object.fromEntries(['marked','katex','playwright','pdf-lib'].map(name=>[name,dependency(name+'/package.json').version])),source_sha256:crypto.createHash('sha256').update(fs.readFileSync(__filename)).digest('hex'),stylesheet_sha256:crypto.createHash('sha256').update(css).digest('hex')},sources:[],math_count:0,outputs:[]};
  docs.clear();sourceCopies.clear();
  chapters=discoverChapters(root).map(load);
  glossary=load(bookRoot+'/notation_and_glossary.md');sources=load(bookRoot+'/source_registry.md');
}
const docs = new Map();
function load(relative) {
  const text = readSource(root,relative).toString('utf8');
  const origin = 'filesystem-candidate';
  const title = text.match(/^title:\s*(.+)$/m)?.[1]?.replace(/^['"]|['"]$/g, '') || path.basename(relative);
  const doc = { relative, text, title, id: slug(path.basename(relative,'.md')), origin };
  docs.set(relative.replace(/\.md$/, ''), doc);
  manifest.sources.push({ path: relative, origin, sha256: crypto.createHash('sha256').update(text).digest('hex') });
  return doc;
}
let currentDoc;
let selectedDocs;
const sourceCopies = new Map();
function safeTarget(target, outputRoot = out) {
  target=path.resolve(target);outputRoot=path.resolve(outputRoot);
  if(target!==outputRoot&&!target.startsWith(outputRoot+path.sep))throw new Error(`Generated target escapes output root: ${target}`);
  let cursor=target;
  while(true){
    try { if(fs.lstatSync(cursor).isSymbolicLink())throw new Error(`Symlinked generated target or ancestor: ${cursor}`); }
    catch(error){if(error.code!=='ENOENT')throw error;}
    const parent=path.dirname(cursor);if(parent===cursor)break;cursor=parent;
  }
  return target;
}
function writeOutput(target, bytes) {
  safeTarget(target);fs.mkdirSync(path.dirname(target),{recursive:true});fs.writeFileSync(target,bytes);
}
const headingCounts = new Map();
let unavailable = [];
function resolveLink(href, wiki = false) {
  if (/^(https?:|mailto:)/i.test(href)) return href;
  const [bare, hash] = href.split('#');
  let rel = bare ? (wiki || /^(knowledge|formalization|evidence|labs)\//.test(bare) ? bare : path.posix.normalize(path.posix.join(path.posix.dirname(currentDoc.relative), bare))) : currentDoc.relative;
  const doc = docs.get(rel.replace(/\.md$/, ''));
  if (doc && selectedDocs.has(doc.relative)) return '#' + doc.id + (hash ? '-' + slug(hash) : '');
  if (!path.extname(rel)) { try { sourcePath(root,rel+'.md'); rel += '.md'; } catch(error) { if(error.code!=='ENOENT')throw error; } }
  let absolute;
  try { absolute=sourcePath(root,rel); } catch(error) {
    if(error.code!=='ENOENT')throw error;
    if(release)throw new Error(`Unresolved source reference: ${href}`);
    const id='unavailable-ref-'+(unavailable.length+1);
    unavailable.push({id,source:currentDoc.relative,target:href});
    return '#'+id;
  }
  if (fs.statSync(absolute).isFile()) {
    const target = 'sources/' + rel;
    if (!sourceCopies.has(target)) sourceCopies.set(target, readSource(root,rel));
    return pathToFileURL(path.join(out,target)).href + (hash ? '#' + hash : '');
  }
  throw new Error(`Reference points to a directory: ${href}`);
}
function math(tex, display) {
  manifest.math_count++;
  // Presentation-only house style; canonical source bytes remain unchanged.
  tex = tex.replace(/\\(begin|end)\{pmatrix\}/g, '\\$1{bmatrix}');
  if (display && tex.startsWith('\\text{support geometry}')) {
    tex='\\begin{aligned}&'+tex.split('\\Rightarrow').join('\\\\ &\\Rightarrow')+'\\end{aligned}';
  } else if (display && /\\operatorname\{(?:HasParametricPowerCauchyFormula|FiniteMatrixMainTheoremStatement)\}/.test(tex) && tex.includes('\\iff')) {
    const [left,right]=tex.split('\\iff');
    tex='\\begin{aligned}&'+left.trim()+'\\\\ &\\iff '+right.trim().replace(/\n/g,'\\\\ &\\qquad ')+'\\end{aligned}';
  }
  return katex.renderToString(tex, { displayMode: display, throwOnError: true, trust: false, strict: false, output: 'htmlAndMathml' });
}
marked.use({ gfm: true, extensions: [
  { name:'displayMath', level:'block', start:src=>src.indexOf('$$'), tokenizer(src) {
    const m=/^\$\$([\s\S]+?)\$\$(?:\n|$)/.exec(src);
    if(m)return {type:'displayMath',raw:m[0],text:m[1].trim()};
  }, renderer:t=>`<div class="math-block">${math(t.text,true)}</div>\n` },
  { name:'inlineMath', level:'inline', start:src=>src.indexOf('$'), tokenizer(src) {
    const m=/^\$(?!\$)((?:\\.|[^$])+?)\$(?!\$)/.exec(src);
    if(m)return {type:'inlineMath',raw:m[0],text:m[1]};
  }, renderer:t=>math(t.text,false) },
  { name:'wikiLink', level:'inline', start:src=>src.indexOf('[['), tokenizer(src) {
    const m=/^\[\[([^\]|]+)(?:\|([^\]]+))?\]\]/.exec(src);
    if(m)return {type:'wikiLink',raw:m[0],href:m[1],text:m[2]||m[1]};
  }, renderer:t=>`<a href="${escape(resolveLink(t.href,true))}">${escape(t.text)}</a>` }
], renderer: {
  heading(token) {
    const explicit=token.text.match(/\s*\{#([^}]+)\}\s*$/);
    const clean=token.text.replace(/\s*\{#[^}]+\}\s*$/, '');
    const base=currentDoc.id+'-'+slug(explicit?explicit[1]:clean);
    const count=(headingCounts.get(base)||0)+1;headingCounts.set(base,count);
    const id=base+(count>1?'-'+count:'');
    const numbered=clean.match(/^(CFT-\d{2}-\d{3})\s+[—–-]\s+(.+)$/);
    const title=numbered?`<span class="theorem-id">${escape(numbered[1])}</span>${marked.parseInline(numbered[2])}`:marked.parseInline(clean);
    return `<h${token.depth} id="${id}">${title}</h${token.depth}>\n`;
  },
  link(token) { return `<a href="${escape(resolveLink(token.href))}">${this.parser.parseInline(token.tokens)}</a>`; },
  paragraph(token) {
    let role='';
    if(token.text.startsWith('Receipt audit:'))role='receipt';
    else if(/^Lean (?:correspondence|scope|kernel|definition|coordinate calculation|checks)[:\s]/.test(token.text))role='evidence-note';
    else if(/^\*\*(?:Theorem|Definition|Lemma|Proposition|Worked theorem)\b/.test(token.text))role='statement';
    return `<p${role?` class="${role}"`:''}>${this.parser.parseInline(token.tokens)}</p>\n`;
  },
  html(token) { return escape(token.text); }
} });

function body(doc) {
  currentDoc=doc;
  let text=doc.text.replace(/^---\n[\s\S]*?\n---\n/, '').replace(/^# .+\n/m, '');
  text=text.replace(/^(?:Book|Part|Previous|Next):.*\n/gm, '').replace(/^Back to the .*\n/m,'');
  try { return marked.parse(text); } catch(error) { throw new Error(`${doc.relative}: ${error.message}`); }
}
function chapter(doc) {
  const n=Number(path.basename(doc.relative).slice(0,2));
  const part=Number(doc.relative.match(/part_(\d+)/)[1]);
  currentDoc=doc;
  const moduleLink=resolveLink(chapterModule(root,n),true);
  return `<article class="chapter" id="${doc.id}"><header class="chapter-head"><div class="eyebrow">Part ${part} · ${parts[part-1]}</div><div class="chapter-number">${String(n).padStart(2,'0')}</div><h1>${escape(doc.title)}</h1><p class="chapter-source"><a href="${escape(moduleLink)}">Lean chapter module</a></p><div class="chapter-rule"></div></header>${body(doc)}</article>`;
}
function editionNote(full) {
  return `<section class="frontmatter edition-note" id="edition"><div class="eyebrow">About this edition</div><h1>A book in progress</h1><p>This reading edition brings the maintained textbook into a quiet, print-oriented format. The mathematical exposition, exercises, motivation, historical context, and Lean references come from the canonical chapter sources.</p><div class="status-note"><p><strong>Working draft · ${date}.</strong> Typesetting is not a new proof certification. Chapter depth and formal correspondence vary. The text marks the boundaries between written arguments, exact Lean statements, and numerical examples.</p><p>This edition uses the source files present in the candidate checkout. Their exact hashes are recorded in the build manifest. No missing chapter is replaced from Git history. Formal verification status is described by the canonical sources; generating this PDF performs no proof compilation.</p></div><p>Square-bracket matrix displays follow the adopted house style. The typesetter adjusts matrix delimiters and breaks a few long displays across lines; it does not alter the canonical mathematical statements.</p><p>Blue links navigate within this PDF or open source references. Local source links point to the accompanying source snapshot. They require a PDF reader that permits local file links. The source files and their exact hashes are listed in <code>build-manifest.json</code>.</p><p class="print-note">Source branch: ${escape(branch)}<br>Checkout HEAD (not a claim of clean source state): ${commit}<br>Text set in Georgia; mathematics set with KaTeX. Generated from repository-owned Markdown.</p></section>`;
}
function contents(list, full) {
  let prior=0;
  const entry=(doc,title,number='')=>`<div class="toc-entry">${number?`<span class="toc-number">${number}</span>`:''}<a href="#${doc.id}">${escape(title)}</a><span class="toc-page" data-target="${doc.id}"></span></div>`;
  const rows=list.map(doc=>{
    const n=Number(path.basename(doc.relative).slice(0,2));
    const part=Number(doc.relative.match(/part_(\d+)/)[1]);
    const label=part!==prior?`<div class="toc-part">Part ${part} · ${parts[part-1]}</div>`:'';
    prior=part;
    return label+entry(doc,doc.title,String(n).padStart(2,'0'));
  }).join('');
  return `<section class="frontmatter contents"><div class="eyebrow">Reader's map</div><h1>Contents</h1>${rows}<div class="toc-part">Reference</div>${entry(glossary,'Notation and glossary')}${full?entry(sources,'Sources and attribution'):''}</section>`;
}
let katexCSSPath;
try { katexCSSPath=require.resolve('katex/dist/katex.min.css'); }
catch { katexCSSPath=requireAtlas.resolve('katex/dist/katex.min.css'); }
const katexCSS=fs.readFileSync(katexCSSPath,'utf8').replace(/url\(([^)]+)\)/g, (_,url)=> {
  const p=path.resolve(path.dirname(katexCSSPath),url.replace(/["']/g,''));
  return `url(data:font/${p.endsWith('.woff2')?'woff2':p.endsWith('.woff')?'woff':'ttf'};base64,${fs.readFileSync(p).toString('base64')})`;
});
const css=fs.readFileSync(path.join(__dirname,'book.css'),'utf8');
function documentHTML(list, full) {
  unavailable=[];headingCounts.clear();
  selectedDocs=new Set([...list,glossary,...(full?[sources]:[])].map(d=>d.relative));
  const title=full?'Crouzeix<br>Foundations':'Objects &amp;<br>Representations';
  const subtitle=full?'From linear algebra to operator theory':'Chapter 1 · A coordinate workshop';
  const cover=`<section class="cover"><div class="eyebrow">Harp mathematical library</div><h1>${title}</h1><p class="subtitle">${subtitle}</p><div class="rule"></div><p class="audience">A proof-oriented textbook for<br>machine-learning researchers</p><div class="cover-math">${math('[T(v)]_C=[T]_{C\\leftarrow B}[v]_B',false)}</div><div class="edition">READING EDITION · ${date}<br>${full?'35 chapters · Six parts':'Expanded exposition · Lean source references'}<br>Working draft</div></section>`;
  const main=list.map(chapter).join('\n');
  const reference=`<section class="appendix" id="${glossary.id}"><div class="eyebrow">Reference</div><h1>Notation and glossary</h1>${body(glossary)}</section>${full?`<section class="appendix" id="${sources.id}"><div class="eyebrow">Reference</div><h1>Sources and attribution</h1>${body(sources)}</section>`:''}`;
  const missing=unavailable.length?`<section class="appendix source-list"><h1>Unresolved source references</h1><p>These references appear in the canonical draft, but their targets are not available in this snapshot. They are retained here rather than replaced by a different source.</p>${unavailable.map(x=>`<p id="${x.id}"><strong>${escape(x.target)}</strong><br>From ${escape(x.source)}</p>`).join('')}</section>`:'';
  return `<!doctype html><html lang="en"><head><meta charset="utf-8"><title>${full?'Crouzeix Foundations':'Objects and Representations'}</title><style>${katexCSS}\n${css}</style></head><body>${cover}${editionNote(full)}${contents(list,full)}${main}${reference}${missing}</body></html>`;
}

async function main() {
  if(process.argv.slice(2).some(arg=>arg!=='--staged'))throw new Error('Usage: node build.cjs [--staged]');
  initialize();
  safeTarget(out);
  fs.mkdirSync(out,{recursive:true});
  const executablePath=process.env.CHROMIUM_EXECUTABLE || chromium.executablePath();
  const browser=await chromium.launch({headless:true,executablePath});
  manifest.generator.browser=await browser.version();
  try {
    for(const [name,list,full] of [['crouzeix-foundations',chapters,true],['chapter-01-objects-and-representations',[chapters[0]],false]]) {
      const html=documentHTML(list,full);
      const htmlPath=path.join(out,name+'.html');
      writeOutput(htmlPath,html);
      const page=await browser.newPage({viewport:{width:582,height:900}});
      // No external resources are needed. Keep printing offline.
      await page.route(/^https?:/,route=>route.abort());
      await page.goto(pathToFileURL(htmlPath).href,{waitUntil:'load'});
      await page.emulateMedia({media:'print'});
      await page.evaluate(()=>document.fonts.ready);
      const session=await page.context().newCDPSession(page);
      await session.send('DOM.enable');await session.send('CSS.enable');
      const {root:dom}=await session.send('DOM.getDocument');
      const {nodeIds}=await session.send('DOM.querySelectorAll',{nodeId:dom.nodeId,selector:'p, h1, h2, code, .katex-html span'});
      const fonts=new Map();
      for(const nodeId of nodeIds.slice(0,300))for(const font of (await session.send('CSS.getPlatformFontsForNode',{nodeId})).fonts)fonts.set(font.postScriptName,font);
      await session.detach();
      manifest.generator.rendered_font_samples ??= {};
      manifest.generator.rendered_font_samples[name]=[...fonts.values()];
      manifest.generator.font_environment=await page.evaluate(()=>({platform:navigator.platform,user_agent:navigator.userAgent,body_stack:getComputedStyle(document.body).fontFamily}));
      const adjustments=await page.evaluate(()=>{
        const scaled=[];
        for(const el of document.querySelectorAll('.katex-display')) {
          const inner=el.querySelector('.katex-html');
          const available=el.clientWidth;
          const actual=Math.max(inner?.scrollWidth||0,el.querySelector('.katex')?.scrollWidth||0);
          if(actual>available+1){ const ratio=available/actual;el.style.fontSize=`${ratio*100}%`;scaled.push({ratio,text:el.textContent.slice(0,100)}); }
        }
        return scaled;
      });
      const pdfPath=path.join(out,name+'.pdf');
      safeTarget(pdfPath);
      const pdfOptions={path:pdfPath,preferCSSPageSize:true,printBackground:true,displayHeaderFooter:true,tagged:true,outline:true,
        headerTemplate:'<div></div>',
        footerTemplate:'<div></div>'};
      let pdf;
      let tocPageMap;
      for(let pass=0;pass<3;pass++) {
        await page.pdf(pdfOptions);
        pdf=await PDFDocument.load(fs.readFileSync(pdfPath));
        const dests=pdf.catalog.lookup(PDFName.of('Dests'),PDFDict);
        const pageNumbers=new Map(pdf.getPages().map((p,i)=>[p.ref.toString(),i+1]));
        tocPageMap=Object.fromEntries(dests.entries().map(([key,value])=>[key.decodeText(),pageNumbers.get(pdf.context.lookup(value,PDFArray).get(0).toString())]));
        const changed=await page.evaluate(map=>{
          let changed=false;
          for(const el of document.querySelectorAll('.toc-page')) {
            const number=map[el.dataset.target];if(!number)throw new Error('Missing PDF contents destination: '+el.dataset.target);
            if(el.textContent!==String(number)){el.textContent=String(number);changed=true;}
          }
          return changed;
        },tocPageMap);
        if(!changed)break;
        if(pass===2)throw new Error('Contents pagination failed to converge');
      }
      writeOutput(htmlPath,(await page.content()).split(pathToFileURL(out+path.sep).href).join(''));
      const runningFont=await pdf.embedFont(StandardFonts.Helvetica);
      const starts=[...list,glossary,...(full?[sources]:[])].map(doc=>({page:tocPageMap[doc.id],title:doc.title})).sort((a,b)=>a.page-b.page);
      for(const [index,pdfPage] of pdf.getPages().entries()) {
        if(index===0)continue;
        pdfPage.pushOperators(beginMarkedContent('Artifact'));
        const number=index+1;
        const section=starts.filter(s=>s.page<=number).at(-1);
        const title=section?.title||(number===2?'About this edition':'Contents');
        const color=rgb(.39,.45,.46);
        pdfPage.drawText(title,{x:51.84,y:pdfPage.getHeight()-29,size:7.3,font:runningFont,color});
        pdfPage.drawLine({start:{x:51.84,y:pdfPage.getHeight()-36},end:{x:pdfPage.getWidth()-51.84,y:pdfPage.getHeight()-36},thickness:.3,color:rgb(.78,.82,.80)});
        const folio=String(number);
        pdfPage.drawText(folio,{x:pdfPage.getWidth()-51.84-runningFont.widthOfTextAtSize(folio,8),y:26,size:8,font:runningFont,color});
        pdfPage.drawText('HARP  /  WORKING DRAFT',{x:51.84,y:26,size:6.3,font:runningFont,color});
        pdfPage.pushOperators(endMarkedContent());
      }
      const sourcePrefix=pathToFileURL(out+path.sep).href;
      let portableSourceLinks=0;
      for(const pdfPage of pdf.getPages()) {
        const annotations=pdfPage.node.lookupMaybe(PDFName.of('Annots'),PDFArray);
        if(!annotations)continue;
        for(const ref of annotations.asArray()) {
          const annotation=pdf.context.lookup(ref,PDFDict);
          const action=annotation.lookupMaybe(PDFName.of('A'),PDFDict);
          if(!action)continue;
          const uri=action.get(PDFName.of('URI'));
          if(!(uri instanceof PDFString || uri instanceof PDFHexString))continue;
          const value=uri.decodeText();
          if(value.startsWith(sourcePrefix)){action.set(PDFName.of('URI'),PDFString.of(value.slice(sourcePrefix.length)));portableSourceLinks++;}
        }
      }
      pdf.setTitle(full?'Crouzeix Foundations':'Objects and Representations');
      pdf.setSubject('Working-draft mathematical reading edition with Lean source references');
      writeOutput(pdfPath,await pdf.save());
      manifest.outputs.push({file:name+'.pdf',pages:pdf.getPageCount(),contents_pages:tocPageMap,portable_source_links:portableSourceLinks,bytes:fs.statSync(pdfPath).size,sha256:crypto.createHash('sha256').update(fs.readFileSync(pdfPath)).digest('hex'),scaled_displays:adjustments,unresolved_references:[...unavailable]});
      console.log(JSON.stringify({file:pdfPath,scaled_displays:adjustments.length}));
      await page.close();
    }
  } finally { await browser.close(); }
  for(const doc of docs.values()) sourceCopies.set('sources/'+doc.relative,Buffer.from(doc.text));
  for(const [relative,bytes] of sourceCopies) {
    const source=relative.slice('sources/'.length);
    if(!readSource(root,source).equals(bytes))throw new Error(`Source changed during build: ${source}`);
    if(process.argv.includes('--staged')) {
      const staged=execFileSync('git',['show',`:${source}`],{cwd:root});
      if(!staged.equals(bytes))throw new Error(`Source differs from staged candidate: ${source}`);
    }
  }
  manifest.staged_candidate_verified=process.argv.includes('--staged');
  manifest.linked_sources=[];
  for(const [relative,bytes] of sourceCopies){const target=path.join(out,relative);writeOutput(target,bytes);manifest.linked_sources.push({path:relative,sha256:crypto.createHash('sha256').update(bytes).digest('hex')});}
  manifest.candidate_sha256=candidateHash(manifest.linked_sources);
  writeOutput(path.join(out,'build-manifest.json'),JSON.stringify(manifest,null,2)+'\n');
}
if(require.main===module)main().catch(error=>{console.error(error);process.exitCode=1;});
module.exports={safeTarget,editionNote, renderForTest(text){selectedDocs=new Set();headingCounts.clear();const prior=release;release=false;try{return body({relative:'knowledge/crouzeix_textbook/fixture.md',id:'fixture',text});}finally{release=prior;}}};
