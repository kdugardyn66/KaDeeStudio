/* mdcraft — slide show (F5): full screen, one slide at a time, keyboard and click
 * navigation, speaker notes (N), black screen (B). */
(function () {
  'use strict';
  const MD = window.MD;
  const ML = window.ML;
  const { h } = MD;
  const root = document.getElementById('md-present');
  const stage = document.getElementById('md-present-stage');
  const count = document.getElementById('md-present-count');
  const notesEl = document.getElementById('md-present-notes');
  const progress = document.getElementById('md-present-progress');
  let slides = [];
  let n = 0;
  let showNotes = false;
  let wasFullscreen = false;
  let hideTimer = null;

  MD.presenting = () => !root.hidden;

  /** Static copies of the rendered slides (charts become pictures). */
  async function collect() {
    const content = document.getElementById('ml-content');
    await (ML.renderPromise || Promise.resolve());
    const wraps = [...content.querySelectorAll('.ml-slide-wrap')];
    const out = [];
    for (const w of wraps) {
      const s = w.querySelector('.ml-slide');
      if (!s) continue;
      const clone = s.cloneNode(true);
      // charts are drawn again at full size in the slide show; other canvases become pictures
      clone.querySelectorAll('.ml-diagram[data-kind="chart"]').forEach((c, i) => { const o = s.querySelectorAll('.ml-diagram[data-kind="chart"]')[i]; c.dataset.src = (o && o.dataset.src) || ''; c.replaceChildren(); c.classList.add('md-live-chart'); });
      const canv = [...s.querySelectorAll('canvas')].filter((c) => !c.closest('.ml-diagram[data-kind="chart"]'));
      const cc = [...clone.querySelectorAll('canvas')];
      canv.forEach((c, i) => { try { const img = h('img.md-present-chart', { src: c.toDataURL('image/png') }); cc[i] && cc[i].replaceWith(img); } catch (e) { /* tainted */ } });
      clone.querySelectorAll('.ml-copy, .ml-slide-num').forEach((e) => e.remove());
      const notes = w.querySelector('.ml-notes');
      out.push({ el: clone, notes: notes ? notes.innerHTML.replace(/<div class="ml-notes-label">[^<]*<\/div>/, '') : '' });
    }
    if (!out.length) {
      // a document without slides: present each top-level section as a slide
      const art = content.cloneNode(true);
      let cur = null;
      for (const node of [...art.children]) {
        if (/^H[12]$/.test(node.tagName) || !cur) { cur = h('div.ml-slide.md-auto-slide'); out.push({ el: cur, notes: '' }); }
        cur.append(node);
      }
    }
    return out;
  }

  function fit() {
    const W = innerWidth, H = innerHeight - (showNotes ? Math.min(220, innerHeight * 0.28) : 0);
    let w = W, hh = W * 9 / 16;
    if (hh > H) { hh = H; w = H * 16 / 9; }
    stage.style.width = w + 'px';
    stage.style.height = hh + 'px';
    stage.style.marginTop = Math.max(0, (H - hh) / 2) + 'px';
  }

  function go(i) {
    if (!slides.length) return;
    n = Math.max(0, Math.min(slides.length - 1, i));
    const s = slides[n];
    const art = h('article.markdown-body.ml-deck.md-present-body.' + [...document.getElementById('ml-content').classList].filter((c) => c.startsWith('theme-')).join('.'));
    art.style.setProperty('--zoom', 1);
    art.append(s.el.cloneNode(true));
    stage.replaceChildren(art);
    count.textContent = (n + 1) + ' / ' + slides.length;
    progress.style.width = ((n + 1) / slides.length * 100) + '%';
    notesEl.innerHTML = s.notes || '<p class="md-hint">No speaker notes for this slide.</p>';
    // re-run the renderers that need a live element (Mermaid SVG is copied; math is copied)
    stage.classList.remove('md-black');
    const chart = ML.plugins.find((p) => p.name === 'chart');
    if (chart) art.querySelectorAll('.md-live-chart').forEach((el) => { chart.load().then(() => chart.render(el)).catch(() => {}); });
  }

  MD.present = async (from) => {
    if (MD.presenting()) return;
    if (MD.view && MD.view() === 'write') MD.setView('split');
    MD.toast('Starting the slide show…', 1200);
    await new Promise((r) => setTimeout(r, 30));
    slides = await collect();
    if (!slides.length) { MD.toast('Nothing to present yet'); return; }
    root.hidden = false;
    document.body.classList.add('md-presenting');
    showNotes = false; notesEl.hidden = true;
    fit();
    go((from || 1) - 1);
    wasFullscreen = false;
    if (MD.isNative()) { MD.native('fullscreen', { on: true }); wasFullscreen = true; }
    else if (document.documentElement.requestFullscreen) { try { await document.documentElement.requestFullscreen(); wasFullscreen = true; } catch (e) { /* not allowed */ } }
    setTimeout(fit, 250);
    root.focus();
    poke();
  };
  function exit() {
    root.hidden = true;
    document.body.classList.remove('md-presenting');
    stage.replaceChildren();
    if (wasFullscreen) {
      if (MD.isNative()) MD.native('fullscreen', { on: false });
      else if (document.fullscreenElement) document.exitFullscreen().catch(() => {});
    }
    // leave the editor on the slide we ended on
    if (MD.isDeck()) MD.editor.goToSlide(n + 1);
  }
  function poke() {
    root.classList.add('md-show-bar');
    clearTimeout(hideTimer);
    hideTimer = setTimeout(() => root.classList.remove('md-show-bar'), 2200);
  }

  document.addEventListener('keydown', (e) => {
    if (!MD.presenting()) return;
    const k = e.key;
    if (/^\d$/.test(k)) { // a slide number, then Enter, jumps there
      jump += k; clearTimeout(jumpTimer); jumpTimer = setTimeout(() => { jump = ''; }, 1500);
      e.preventDefault(); e.stopPropagation(); return;
    }
    if (k === 'Enter' && jump) { go(+jump - 1); jump = ''; }
    else if (['ArrowRight', 'ArrowDown', 'PageDown', ' ', 'Enter'].includes(k)) go(n + 1);
    else if (['ArrowLeft', 'ArrowUp', 'PageUp', 'Backspace'].includes(k)) go(n - 1);
    else if (k === 'Home') go(0);
    else if (k === 'End') go(slides.length - 1);
    else if (k === 'Escape') exit();
    else if (k === 'n' || k === 'N') { showNotes = !showNotes; notesEl.hidden = !showNotes; fit(); }
    else if (k === 'b' || k === 'B' || k === '.') stage.classList.toggle('md-black');
    else if (k === 'F5' || (e.ctrlKey || e.metaKey)) { /* swallow */ }
    else return;
    e.preventDefault(); e.stopPropagation();
  }, true);
  let jump = '', jumpTimer = null;
  root.addEventListener('click', (e) => {
    const b = e.target.closest('[data-p]');
    if (b) {
      const p = b.dataset.p;
      if (p === 'prev') go(n - 1); else if (p === 'next') go(n + 1); else if (p === 'exit') exit();
      else if (p === 'notes') { showNotes = !showNotes; notesEl.hidden = !showNotes; fit(); }
      return;
    }
    if (e.target.closest('a, video, audio, iframe, .ml-molstar, #md-present-notes')) return;
    if (e.clientX < innerWidth * 0.25) go(n - 1); else go(n + 1);
  });
  root.addEventListener('mousemove', poke);
  root.addEventListener('wheel', (e) => { if (Math.abs(e.deltaY) > 20) go(n + (e.deltaY > 0 ? 1 : -1)); }, { passive: true });
  addEventListener('resize', () => { if (MD.presenting()) fit(); });
  document.addEventListener('fullscreenchange', () => { if (!document.fullscreenElement && MD.presenting() && !MD.isNative()) { wasFullscreen = false; } });
})();
