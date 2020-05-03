## Type away

Goal: be able to practice typing with learning materials, articles, code etc. 
---

Three different pieces:

- Parser: processes keyboard input, communicates with Checker.
- Checker: processes input from Parser, traverses source string, compares input with source. When done tells Parser to stop. Until done, communicates outcome of input to Renderer. 
- Renderer: processes input from Checker, draws on terminal.

