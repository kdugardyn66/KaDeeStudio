/* mdcraft — UI primitives: element builder, modal dialogs, popup menus, the native bridge. */
(function () {
  'use strict';
  const MD = window.MD = window.MD || {};
  const ML = window.ML;

  // ------------------------------------------------------------------ element builder
  /** h('div.cls#id', {attrs}, children…) — children may be strings, nodes, arrays or null. */
  function h(spec, attrs, ...kids) {
    const m = /^([a-z0-9-]+)?((?:[.#][\w-]+)*)$/i.exec(spec) || [];
    const el = document.createElement(m[1] || 'div');
    (m[2] || '').replace(/([.#])([\w-]+)/g, (_, k, v) => { if (k === '.') el.classList.add(v); else el.id = v; });
    if (attrs && (typeof attrs !== 'object' || attrs instanceof Node || Array.isArray(attrs))) { kids.unshift(attrs); attrs = null; }
    for (const [k, v] of Object.entries(attrs || {})) {
      if (v == null || v === false) continue;
      if (k.startsWith('on') && typeof v === 'function') el.addEventListener(k.slice(2), v);
      else if (k === 'class') el.className += (el.className ? ' ' : '') + v;
      else if (k === 'style' && typeof v === 'object') Object.assign(el.style, v);
      else if (k === 'html') el.innerHTML = v;
      else if (k === 'dataset') Object.assign(el.dataset, v);
      else if (k in el && k !== 'list' && k !== 'form') { try { el[k] = v; } catch (e) { el.setAttribute(k, v); } }
      else el.setAttribute(k, v === true ? '' : v);
    }
    const add = (k) => { if (k == null || k === false) return; if (Array.isArray(k)) k.forEach(add); else el.append(k instanceof Node ? k : document.createTextNode(String(k))); };
    kids.forEach(add);
    return el;
  }
  MD.h = h;
  MD.esc = (s) => String(s ?? '').replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
  MD.icon = (name, size) => { const s = h('span.md-ic'); s.innerHTML = window.MDIcons.icon(name, size); return s; };
  MD.toast = (msg, ms) => ML.toast(msg, ms);
  MD.isMac = /Mac|iPhone|iPad/.test(navigator.platform);
  MD.modKey = MD.isMac ? '⌘' : 'Ctrl+';
  MD.keyLabel = (k) => (k || '').replace(/Mod\+/g, MD.isMac ? '⌘' : 'Ctrl+').replace(/Shift\+/g, MD.isMac ? '⇧' : 'Shift+').replace(/Alt\+/g, MD.isMac ? '⌥' : 'Alt+');

  // ------------------------------------------------------------------ server
  MD.api = async (url, body) => {
    const r = await fetch(url, body === undefined ? { cache: 'no-store' } : { method: 'POST', body: JSON.stringify(body), headers: { 'Content-Type': 'application/json' } });
    let v = null;
    try { v = await r.json(); } catch (e) { /* not json */ }
    if (!r.ok) throw new Error((v && v.error) || ('HTTP ' + r.status));
    return v;
  };

  // ------------------------------------------------------------------ native bridge (wry IPC)
  const pending = new Map();
  let nextId = 1;
  MD.isNative = () => !!(window.ipc && typeof window.ipc.postMessage === 'function');
  /** Calls the desktop shell; resolves with its answer (null when cancelled). */
  MD.native = (cmd, args) => new Promise((resolve, reject) => {
    if (!MD.isNative()) { reject(new Error('not in the desktop app')); return; }
    const id = nextId++;
    pending.set(id, { resolve, reject });
    window.ipc.postMessage(JSON.stringify(Object.assign({ cmd, id }, args || {})));
    if (['set-title', 'quit', 'open-external', 'fullscreen', 'new-window', 'ready', 'zoom', 'theme', 'reveal'].includes(cmd)) { pending.delete(id); resolve(null); }
  });
  /** The shell answers here (`window.__mdReply(id, value, error)`). */
  window.__mdReply = (id, value, error) => {
    const p = pending.get(id);
    if (!p) return;
    pending.delete(id);
    if (error) p.reject(new Error(error)); else p.resolve(value);
  };
  ML.native = MD.native;
  ML.isNative = MD.isNative;

  // ------------------------------------------------------------------ menus / popups
  let openMenu = null;
  /** Closes the open popup menu. */
  MD.closeMenu = () => { if (openMenu) { const m = openMenu; openMenu = null; m.el.remove(); m.onClose && m.onClose(); } };
  /**
   * Shows `content` (a node) as a popup under `anchor` (a node or {x, y}).
   */
  MD.popup = (anchor, content, opts) => {
    MD.closeMenu();
    const el = h('div.md-popup.md-menu', content);
    if (opts && opts.cls) el.classList.add(...opts.cls.split(' '));
    document.getElementById('md-menu-root').append(el);
    const r = anchor instanceof Node ? anchor.getBoundingClientRect() : { left: anchor.x, right: anchor.x, top: anchor.y, bottom: anchor.y };
    const W = innerWidth, H = innerHeight;
    const rect = el.getBoundingClientRect();
    let x = r.left, y = r.bottom + 2;
    if (x + rect.width > W - 8) x = Math.max(8, W - rect.width - 8);
    if (y + rect.height > H - 8) y = Math.max(8, (r.top - rect.height - 2 > 8 ? r.top - rect.height - 2 : H - rect.height - 8));
    el.style.left = x + 'px'; el.style.top = y + 'px';
    openMenu = { el, anchor, onClose: opts && opts.onClose };
    MDIcons.fill(el);
    const first = el.querySelector('button:not([disabled]), input');
    if (first && !(opts && opts.noFocus)) setTimeout(() => first.focus(), 0);
    return el;
  };
  /** A menu of items: {label, icon, cmd|run, shortcut, checked, disabled, sep, hint}. */
  MD.menu = (anchor, items, opts) => {
    const list = h('div.md-menu-list', { role: 'menu' });
    for (const it of items) {
      if (!it) continue;
      if (it.sep) { list.append(h('div.md-menu-sep')); continue; }
      if (it.header) { list.append(h('div.md-menu-header', it.header)); continue; }
      const b = h('button.md-menu-item', { role: 'menuitem', disabled: !!it.disabled, title: it.hint || '' },
        h('span.md-menu-check', it.checked ? MD.icon('check', 14) : null),
        it.icon ? MD.icon(it.icon, 18) : h('span.md-ic'),
        h('span.md-menu-label', it.label, it.sub ? h('small', it.sub) : null),
        h('span.md-menu-key', MD.keyLabel(it.shortcut || '')));
      if (it.preview) b.prepend(it.preview);
      b.addEventListener('click', () => { MD.closeMenu(); if (it.run) it.run(); else if (it.cmd) MD.run(it.cmd, it.args); });
      list.append(b);
    }
    list.addEventListener('keydown', (e) => {
      const items = [...list.querySelectorAll('.md-menu-item:not([disabled])')];
      const i = items.indexOf(document.activeElement);
      if (e.key === 'ArrowDown') { e.preventDefault(); items[(i + 1) % items.length]?.focus(); }
      else if (e.key === 'ArrowUp') { e.preventDefault(); items[(i - 1 + items.length) % items.length]?.focus(); }
    });
    return MD.popup(anchor, list, opts);
  };
  document.addEventListener('mousedown', (e) => {
    if (openMenu && !openMenu.el.contains(e.target) && !(openMenu.anchor instanceof Node && openMenu.anchor.contains(e.target))) MD.closeMenu();
  }, true);
  document.addEventListener('keydown', (e) => { if (e.key === 'Escape' && openMenu) { e.stopPropagation(); MD.closeMenu(); } }, true);
  addEventListener('resize', () => MD.closeMenu());

  // ------------------------------------------------------------------ modal dialogs
  const stack = [];
  /**
   * Opens a dialog. opts: {title, body (node), buttons: [{label, value, primary, danger}], width,
   * onKey}. Resolves with the clicked button's value (or the value passed to close()), null on Esc.
   */
  MD.dialog = (opts) => new Promise((resolve) => {
    MD.closeMenu();
    const prevFocus = document.activeElement;
    const close = (v) => {
      const i = stack.indexOf(dlg); if (i >= 0) stack.splice(i, 1);
      back.remove();
      if (prevFocus && prevFocus.focus && document.contains(prevFocus)) prevFocus.focus();
      resolve(v === undefined ? null : v);
    };
    const btns = h('div.md-dlg-buttons');
    for (const b of opts.buttons || [{ label: 'OK', value: true, primary: true }]) {
      const el = h('button.md-btn', { class: (b.primary ? 'md-primary' : '') + (b.danger ? ' md-danger' : '') }, b.label);
      el.addEventListener('click', async () => {
        if (b.validate) { const ok = await b.validate(); if (!ok) return; }
        close(typeof b.value === 'function' ? b.value() : b.value);
      });
      if (b.left) { el.classList.add('md-left'); }
      btns.append(el);
    }
    const dlg = h('div.md-dialog', { role: 'dialog', 'aria-modal': 'true', style: { width: opts.width ? opts.width + 'px' : null } },
      h('div.md-dlg-head', h('div.md-dlg-title', opts.title || ''), h('button.md-icon-btn', { title: 'Close (Esc)', onclick: () => close(null) }, MD.icon('x', 16))),
      h('div.md-dlg-body', opts.body || ''),
      (opts.buttons === null) ? null : btns);
    const back = h('div.md-modal-back', dlg);
    back.addEventListener('mousedown', (e) => { if (e.target === back && opts.dismiss !== false) close(null); });
    dlg.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); close(null); }
      else if (e.key === 'Enter' && !e.shiftKey && e.target.tagName !== 'TEXTAREA' && !(e.target.isContentEditable) && e.target.tagName !== 'BUTTON') {
        const p = btns.querySelector('.md-primary'); if (p) { e.preventDefault(); p.click(); }
      } else if (opts.onKey) opts.onKey(e, close);
    });
    document.getElementById('md-modal-root').append(back);
    MDIcons.fill(dlg);
    stack.push(dlg);
    dlg.close = close;
    if (opts.onOpen) opts.onOpen(dlg, close);
    setTimeout(() => { const f = dlg.querySelector('[autofocus], input, textarea, select, .md-dlg-body button') || btns.querySelector('.md-primary'); f && f.focus(); if (f && f.select && f.tagName === 'INPUT') f.select(); }, 0);
  });
  MD.dialogOpen = () => stack.length > 0;

  MD.alert = (title, msg) => MD.dialog({ title, body: h('p.md-dlg-text', msg), buttons: [{ label: 'OK', value: true, primary: true }] });
  MD.confirm = (title, msg, ok, cancel) => MD.dialog({ title, body: h('p.md-dlg-text', msg), buttons: [{ label: cancel || 'Cancel', value: false }, { label: ok || 'OK', value: true, primary: true }] });
  /** A one-field prompt. Resolves with the text or null. */
  MD.prompt = async (title, label, value, opts) => {
    const input = h('input.md-input', { value: value || '', placeholder: (opts && opts.placeholder) || '', spellcheck: false });
    const v = await MD.dialog({ title, body: h('label.md-field', h('span', label), input), buttons: [{ label: 'Cancel', value: null }, { label: (opts && opts.ok) || 'OK', value: () => input.value, primary: true }] });
    return v;
  };

  // ------------------------------------------------------------------ form helpers
  MD.field = (label, control, hint) => h('label.md-field', h('span', label), control, hint ? h('small.md-hint', hint) : null);
  MD.select = (options, value) => {
    const s = h('select.md-input');
    for (const o of options) { const [v, l] = Array.isArray(o) ? o : [o, o]; s.append(h('option', { value: v, selected: String(v) === String(value) }, l)); }
    return s;
  };
  MD.checkbox = (label, checked, hint) => {
    const c = h('input', { type: 'checkbox', checked: !!checked });
    const l = h('label.md-check', c, h('span', label, hint ? h('small.md-hint', hint) : null));
    l.input = c;
    return l;
  };
  MD.fmtDate = (secs) => {
    if (!secs) return '';
    const d = new Date(secs * 1000), now = new Date();
    const same = d.toDateString() === now.toDateString();
    const y = new Date(now); y.setDate(now.getDate() - 1);
    const time = d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    if (same) return 'Today ' + time;
    if (d.toDateString() === y.toDateString()) return 'Yesterday ' + time;
    return d.toLocaleDateString([], { day: 'numeric', month: 'short', year: d.getFullYear() === now.getFullYear() ? undefined : 'numeric' });
  };
  MD.fmtSize = (n) => (n < 1024 ? n + ' B' : n < 1048576 ? (n / 1024).toFixed(1) + ' KB' : (n / 1048576).toFixed(1) + ' MB');
  MD.basename = (p) => String(p || '').split(/[\\/]/).pop();
  MD.dirname = (p) => { const s = String(p || ''); const i = Math.max(s.lastIndexOf('/'), s.lastIndexOf('\\')); return i > 0 ? s.slice(0, i) : s.slice(0, i + 1); };
  MD.join = (dir, name) => (dir.endsWith('/') || dir.endsWith('\\') ? dir + name : dir + (MD.app && MD.app.sep || '/') + name);
})();
