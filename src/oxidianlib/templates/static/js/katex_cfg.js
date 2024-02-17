document.addEventListener("DOMContentLoaded",
    function () {
      renderMathInElement(document.body,
        {
          delimiters:
            [
              {left: "$$", right: "$$", display: true},
              {left: "\\[", right: "\\]", display: true},
              {left: "$", right: "$", display: false},
              {left: "\\(", right: "\\)", display: false}
            ],
          throwOnError: false, 
          trust: true, 
          strict: false,
          macros:
            { 
              "\\eqref": "\\href{###1}{(\\text{#1})}", // Include macros for equation referencing 
              "\\ref": "\\href{###1}{\\text{#1}}",
              "\\label": "\\hrefId{#1}",
              {{PRMBL}}
            },
        }
      );
    }
);
