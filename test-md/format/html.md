# Format HTML

## Rule: Remove useless spaces, tabs, or newlines inside HTML tags

HTML should be minified. Remove all useless spaces, tabs, or newlines inside HTML tags. It should not change the original HTML tags, attributes, or values.

<!-- Input: -->

<p align="center">
<img src="https://img.shields.io/badge/Markdown-000000?style=for-the-badge&logo=markdown&logoColor=white" alt="Markdown" /></p>

<!-- Correct Output should be: -->

<p align="center">
<img src="https://img.shields.io/badge/Markdown-000000?style=for-the-badge&logo=markdown&logoColor=white" alt="Markdown" /></p>
