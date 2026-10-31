# KaTeX Math Formatting Examples

This file demonstrates KaTeX and LaTeX mathematical notation preserved by agent-md formatter.

## Inline Math

The linear equation is $y = ax + b$, where $a \neq 0$.

Multiplication with asterisks is preserved: $y = a*x + b*c$ and $a * b * c$.

Subscripts with underscores are preserved: $x_{1} + x_{2} = y$ and $\alpha_{min} + \beta_{max} = \gamma_0$.

Exponents and subscripts combined: $x_1^2 + x_2^2 = r^2$.

Comparison operators are not treated as HTML tags: $0 < x < 1$ and $x < y > z$.

Multiple spaces inside math spans are preserved: $a  +  b = c$.

## Display Math

Single-line display math notation:

$$y = ax + b$$

Display math with subscripts and products:

$$x_{1} + x_{2} = a * b * c$$

Multi-line display math block with equation alignment:

$$
\begin{aligned}
x_{1} + x_{2} &= a * b * c \\
y &= d + e
\end{aligned}
$$

Multi-line matrix notation:

$$
\begin{pmatrix}
a & b \\
c & d
\end{pmatrix}
$$

## Math in Tables

| Name | Formula | Notes |
|---|---|---|
| Linear | $y = ax + b$ | First order |
| Quadratic | $y = ax^2 + bx + c$ | Second order |
| Product | $a * b * c$ | Asterisks preserved |
| Subscript | $x_{1} + x_{2}$ | Underscores preserved |
| Magnitude | $|x| < 1$ | Unescaped pipe |

## Math in Lists

- Linear model: $y = ax + b$
- Quadratic equation: $y = ax^2 + bx + c$
- Summation formula: $\sum_{i=1}^{n} x_i$
- Vector notation: $\vec{v} = (x, y, z)$

## Math in Blockquotes

>Formula definition: $y = ax + b$
>Derivative rule: $\frac{d}{dx}(ax + b) = a$
