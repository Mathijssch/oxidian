MathJax = {
    tex: {
      inlineMath: [['$', '$'], ['\\(', '\\)']], 
      displayMath: [['$$', '$$'], ['\\[', '\\]']],
      tags: 'ams',
      packages: {'[+]': ['mathtools']}, 
      macros: {
          {{PRMBL}} 
      }
    },
    svg: {
      fontCache: 'global'
    },
    loader: {load: ['ui/lazy']}
};

//MathJax.Hub.Config({
//  "fast-preview": {disabled: true},
//  tex2jax: {preview: "none"}
//});
