# Third-party notices

## Bundled in the binaries (`crates/mdcraft-core/web/`)

| Component | Used for | License |
|---|---|---|
| [CodeMirror 6](https://codemirror.net) (bundled by `tools/editor-bundle`) | the editor | MIT |
| [Mermaid](https://github.com/mermaid-js/mermaid) | diagrams | MIT |
| [KaTeX](https://github.com/KaTeX/KaTeX) (+ mhchem) | math | MIT |
| [MathJax](https://github.com/mathjax/MathJax) | math (alternative engine) | Apache-2.0 |
| [highlight.js](https://github.com/highlightjs/highlight.js) | code highlighting | BSD-3-Clause |
| [morphdom](https://github.com/patrick-steele-idem/morphdom) | flicker-free updates | MIT |
| [Chart.js](https://github.com/chartjs/Chart.js) | charts | MIT |
| [Vega](https://github.com/vega/vega), [Vega-Lite](https://github.com/vega/vega-lite), [Vega-Embed](https://github.com/vega/vega-embed) | charts | BSD-3-Clause |
| [Plotly.js](https://github.com/plotly/plotly.js) | interactive charts | MIT |
| [Viz.js](https://github.com/mdaines/viz-js) (Graphviz) | graphs | MIT (Graphviz: EPL-1.0) |
| [WaveDrom](https://github.com/wavedrom/wavedrom) | timing diagrams | MIT |
| [flowchart.js](https://github.com/adrai/flowchart.js) + Raphaël | flowcharts | MIT |
| [SmilesDrawer](https://github.com/reymond-group/smilesDrawer) | chemical structures | MIT |
| [Mol*](https://github.com/molstar/molstar) | 3D molecules | MIT |
| Inter, Source Sans 3, Source Serif 4, JetBrains Mono | interface and document fonts (WOFF2) | SIL Open Font License 1.1 (`web/fonts/OFL-*.txt`) |

## Rust crates (compiled in)

comrak (BSD-2-Clause); tao, wry, muda (Apache-2.0 OR MIT); rfd (MIT); serde, serde_json, ureq, flate2,
libc, png, diffy (MIT OR Apache-2.0); and their dependencies (`cargo tree`).

## Used when installed (not bundled)

Java + PlantUML, ffmpeg, Python and other interpreters for code chunks, WebKitGTK (Linux).
