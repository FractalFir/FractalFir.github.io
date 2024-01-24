function cil() {
  const regex = hljs.regex;
     const KEYWORDS = ["locals","method","public","static","private","hidebysig"];
   var types = ["int8","int16","int32","int64","nint","uint8","uint16","uint32","uint64","nuint","bool","char","float32","float64","void","valuetype"];
   var short_types = ["i1","i2","i4","i8","s"];
   var ops = ["conv","ldloc","ldc","add","stloc"];

   var keywords = {
    keyword: KEYWORDS,
    $pattern:/(\w+)/,
    relevance:10,
    literal: types.concat(ops).concat(short_types),
  }
  
  var INLINE_COMMENT = hljs.COMMENT('//', '[^\\\\]$');
  return {
    unicodeRegex: true,
    aliases: ['cil','common_intermediate_language'],
    keywords: keywords,
    contains: [
      INLINE_COMMENT, // single-line comments
      hljs.C_BLOCK_COMMENT_MODE, // comment blocks
    ]
  };
}

hljs.registerLanguage('cil',cil);