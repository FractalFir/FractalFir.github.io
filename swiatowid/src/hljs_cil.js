function cil() {
  const regex = hljs.regex;
     const KEYWORDS = ["locals","method","public","static","private","hidebysig","assembly","class","extends","field","entrypoint","line"];
   var types = ["int8","int16","int32","int64","nint","uint8","uint16","uint32","uint64","nuint","bool","char","float32","float64","void","uint","int","valuetype","native"];
   var short_types = ["i1","i2","i4","i8","s","u1","u2","u4","u8"];
   var ops = ["conv","ldloc","ldc","add","stloc","ret","call","ldarg","ldarga","sizeof","mul","ldobj","div","blt","br",".rs",".s","calli"];

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
function mir() {
  const regex = hljs.regex;
     const KEYWORDS = ["switchInt","unreachable","return","unwind","continue"];

   var vars = ["bb0","bb1","bb2","bb3","bb4","_0","_1","_2"];

   var keywords = {
    keyword: KEYWORDS,
    $pattern:/(\w+)/,
    relevance:10,
    literal: vars,
  }
  
  var INLINE_COMMENT = hljs.COMMENT('//', '[^\\\\]$');
  return {
    unicodeRegex: true,
    aliases: ['mir'],
    keywords: keywords,
    contains: [
      INLINE_COMMENT, // single-line comments
      hljs.C_BLOCK_COMMENT_MODE, // comment blocks
    ]
  };
}

hljs.registerLanguage('cil',cil);
hljs.registerLanguage('mir',mir);