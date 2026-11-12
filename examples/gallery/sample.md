# mdhtml2gpui

Paragraphs carry *emphasis*, **strong**, ~~strike~~, `inline code`, ==highlights==, [links](https://answer.ai), H~2~O, x^2^ and a footnote[^gpui].
A hard break follows.\
Raw output for another format, `\LaTeX`{=latex}, and inline maths, \(e^{i\pi} + 1 = 0\), show their source.

## Lists

1. First
2. Second, with a nested list:
   - nested bullet
   - another

- [x] A finished task
- [ ] An open task

> A block quote with **strong** text.

```python
def greet(name): return f"Hello, {name}"
```

---

| Name     | Kind      | Value |
|:---------|:---------:|------:|
| alpha    | code      | 1.5   |
| beta     | note      | 22    |
| gamma    | prompt    | 333   |

Term
: Its definition.

::: details
### Click to open

The body of a collapsible panel.
:::

::: {.callout-tip}
### A tip

Callouts get a coloured edge.
:::

::: bibliography
Howard and Gugger {year=2020}
: Deep learning for coders with fastai and PyTorch.
:::

![A blue dot](data:image/svg+xml;utf8,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20width%3D%2248%22%20height%3D%2248%22%3E%3Ccircle%20cx%3D%2224%22%20cy%3D%2224%22%20r%3D%2220%22%20fill%3D%22%233b82f6%22%2F%3E%3C%2Fsvg%3E)

\[
\int_0^1 x\,dx = \tfrac12
\]

<div>Raw HTML shows its source.</div>

[^gpui]: Footnotes collect at the end.
