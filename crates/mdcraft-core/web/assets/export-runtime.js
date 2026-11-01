/* mdcraft — finishes an exported page in the reader's browser (mdcraft-cli export):
 * Mermaid diagrams, math, code highlighting and Chart.js charts, with libraries from jsDelivr. */
(function () {
  'use strict';
  var CDN = 'https://cdn.jsdelivr.net/npm/';
  function load(src) { return new Promise(function (res, rej) { var s = document.createElement('script'); s.src = src; s.onload = res; s.onerror = rej; document.head.appendChild(s); }); }
  function css(href) { var l = document.createElement('link'); l.rel = 'stylesheet'; l.href = href; document.head.appendChild(l); }
  function relaxed(t) { try { return JSON.parse(t); } catch (e) { return (new Function('return (' + t + ')'))(); } }
  var doc = document;
  var diagrams = doc.querySelectorAll('.ml-diagram[data-kind="mermaid"]');
  if (diagrams.length) load(CDN + 'mermaid@11.17.2/dist/mermaid.min.js').then(function () {
    window.mermaid.initialize({ startOnLoad: false, theme: 'default', securityLevel: 'strict' });
    diagrams.forEach(function (el, i) {
      var src = (el.querySelector('pre.ml-src') || el).textContent;
      window.mermaid.render('mx' + i, src).then(function (r) { el.innerHTML = r.svg; }).catch(function () {});
    });
  });
  var math = doc.querySelectorAll('[data-math-style]');
  if (math.length) { css(CDN + 'katex@0.16.47/dist/katex.min.css'); load(CDN + 'katex@0.16.47/dist/katex.min.js').then(function () { return load(CDN + 'katex@0.16.47/dist/contrib/mhchem.min.js'); }).then(function () {
    math.forEach(function (el) { try { window.katex.render(el.textContent, el, { displayMode: el.dataset.mathStyle === 'display', throwOnError: false }); } catch (e) {} });
  }); }
  var code = doc.querySelectorAll('pre.ml-code code[class*="language-"]');
  if (code.length) load(CDN + '@highlightjs/cdn-assets@11.12.0/highlight.min.js').then(function () { code.forEach(function (c) { try { window.hljs.highlightElement(c); } catch (e) {} }); });
  var charts = doc.querySelectorAll('.ml-diagram[data-kind="chart"]');
  if (charts.length) load(CDN + 'chart.js@4.5.1/dist/chart.umd.min.js').then(function () {
    charts.forEach(function (el) {
      try { var cfg = relaxed((el.querySelector('pre.ml-src') || el).textContent); var box = document.createElement('div'); box.className = 'ml-chart-box'; var cv = document.createElement('canvas'); box.appendChild(cv); el.replaceChildren(box); new window.Chart(cv, cfg); } catch (e) {}
    });
  });
})();
