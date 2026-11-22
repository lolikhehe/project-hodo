// ESM entry for the local Lit-substrate twin. Two static deps + one dynamic.
import { MARK, fillParts } from './chunk-view.js';
import { LABEL, BODY } from './chunk-model.js';

var TEMPLATE = '<style>' +
  'header{background:#102080;color:#ffffff;font-size:20px;padding:6px 8px}' +
  'p{font-size:16px;margin:6px 8px}' +
  '</style>' +
  '<header><!--?lit$123$--></header>' +
  '<p><!--?lit$123$--></p>' +
  '<p class="dyn">item: <!--?lit$123$--></p>';

var PENDING = '…loading';

// Hand-rolled custom element; no Lit runtime, just the substrate APIs.
class XLitCard extends HTMLElement {
  connectedCallback() {
    if (this.shadowRoot) return;
    var root = this.attachShadow({ mode: 'open' });

    var t = document.createElement('template');
    t.innerHTML = TEMPLATE;
    root.appendChild(document.importNode(t.content, true));

    var sheet = new CSSStyleSheet();
    sheet.replaceSync('.dyn{color:#00806b;font-size:16px}');
    root.adoptedStyleSheets = [sheet];

    var found = fillParts(root, [
      this.getAttribute('data-title') || LABEL,
      this.getAttribute('data-body') || BODY,
      PENDING
    ]);
    this.__slots = found.slots;

    var probe = window.__hodoProbe || (window.__hodoProbe = {});
    probe.markers = (probe.markers || 0) + found.positions.length;
    probe.filled = (probe.filled || 0) + found.slots.length;
  }
}

// Upgrades both <x-lit-card> tags already present in index.html.
customElements.define('x-lit-card', XLitCard);

var card = document.querySelector('x-lit-card');
import('./chunk-dynamic.js').then(function (chunk) {
  if (card && card.__slots && card.__slots[2]) {
    card.__slots[2].nodeValue = chunk.ITEM;
  }
  var probe = window.__hodoProbe || {};
  document.getElementById('probe').textContent =
    'PROBE markers=' + probe.markers + ' filled=' + probe.filled +
    ' dyn=' + (chunk.ITEM ? 'OK' : 'MISS');
});
