'use strict';

function readingEditions(chapters) {
  if(chapters.length!==36)throw new Error('Reading editions require 36 chapters');
  return [
    {name:'crouzeix-foundations',list:chapters,full:true,
      title:'Crouzeix Foundations',coverTitle:'Crouzeix<br>Foundations',
      subtitle:'From linear algebra to operator theory'},
    {name:'chapter-01-objects-and-representations',list:[chapters[0]],full:false,
      title:'Objects and Representations',coverTitle:'Objects &amp;<br>Representations',
      subtitle:'Chapter 1 · A coordinate workshop'},
    {name:'chapter-36-harp-finite-horizon-proof',list:[chapters[35]],full:false,
      title:'The Harp finite-horizon proof',coverTitle:'The Harp<br>finite-horizon proof',
      subtitle:'Chapter 36 · Exact finite certificates and the constant two'},
  ];
}

module.exports={readingEditions};
