import{t as e}from"./defineProperty-BbfpZ9Tg.js";var t=class extends Event{constructor(t,{oldState:n=``,newState:r=``,...i}={}){super(t,i),e(this,`oldState`,void 0),e(this,`newState`,void 0),this.oldState=String(n||``),this.newState=String(r||``)}},n=new WeakMap;function r(e,r,i){n.set(e,setTimeout(()=>{n.has(e)&&e.dispatchEvent(new t(`toggle`,{cancelable:!1,oldState:r,newState:i}))},0))}var i=globalThis.ShadowRoot||function(){},a=globalThis.HTMLDialogElement||function(){},o=new WeakMap,s=new WeakMap,c=new WeakMap,l=new WeakMap;function u(e){return l.get(e)||`hidden`}var d=new WeakMap;function f(e){return[...e].pop()}function p(e){let t=e.popoverTargetElement;if(!(t instanceof HTMLElement))return;let n=u(t);e.popoverTargetAction===`show`&&n===`showing`||e.popoverTargetAction===`hide`&&n===`hidden`||(n===`showing`?O(t,!0,!0):m(t,!1)&&(d.set(t,e),D(t)))}function m(e,t){return!(e.popover!==`auto`&&e.popover!==`manual`&&e.popover!==`hint`||!e.isConnected||t&&u(e)!==`showing`||!t&&u(e)!==`hidden`||e instanceof a&&e.hasAttribute(`open`)||document.fullscreenElement===e)}function h(e){if(!e)return 0;let t=s.get(document)||new Set,n=c.get(document)||new Set;return n.has(e)?[...n].indexOf(e)+t.size+1:t.has(e)?[...t].indexOf(e)+1:0}function g(e){let t=b(e),n=x(e);return h(t)>h(n)?t:n}function _(e){let t,n=c.get(e)||new Set,r=s.get(e)||new Set,i=n.size>0?n:r.size>0?r:null;return i?(t=f(i),t.isConnected?t:(i.delete(t),_(e))):null}function v(e){for(let t of e||[])if(!t.isConnected)e.delete(t);else return t;return null}function y(e){return typeof e.getRootNode==`function`?e.getRootNode():e.parentNode?y(e.parentNode):e}function b(e){for(;e;){if(e instanceof HTMLElement&&e.popover===`auto`&&l.get(e)===`showing`)return e;if(e=e instanceof Element&&e.assignedSlot||e.parentElement||y(e),e instanceof i&&(e=e.host),e instanceof Document)return}}function x(e){for(;e;){let t=e.popoverTargetElement;if(t instanceof HTMLElement)return t;if(e=e.parentElement||y(e),e instanceof i&&(e=e.host),e instanceof Document)return}}function S(e,t){let n=new Map,r=0;for(let e of t||[])n.set(e,r),r+=1;n.set(e,r),r+=1;let i=null;function a(t){if(!t)return;let r=!1,a=null,o=null;for(;!r;){if(a=b(t)||null,a===null||!n.has(a))return;(e.popover===`hint`||a.popover===`auto`)&&(r=!0),r||(t=a.parentElement)}o=n.get(a),(i===null||n.get(i)<o)&&(i=a)}return a(e.parentElement||y(e)),i}function C(e){return e.hidden||e instanceof i||(e instanceof HTMLButtonElement||e instanceof HTMLInputElement||e instanceof HTMLSelectElement||e instanceof HTMLTextAreaElement||e instanceof HTMLOptGroupElement||e instanceof HTMLOptionElement||e instanceof HTMLFieldSetElement)&&e.disabled||e instanceof HTMLInputElement&&e.type===`hidden`||e instanceof HTMLAnchorElement&&e.href===``?!1:typeof e.tabIndex==`number`&&e.tabIndex!==-1}function w(e){if(e.shadowRoot&&e.shadowRoot.delegatesFocus!==!0)return null;let t=e;t.shadowRoot&&(t=t.shadowRoot);let n=t.querySelector(`[autofocus]`);if(n)return n;{let e=t.querySelectorAll(`slot`);for(let t of e){let e=t.assignedElements({flatten:!0});for(let t of e)if(t.hasAttribute(`autofocus`))return t;else if(n=t.querySelector(`[autofocus]`),n)return n}}let r=e.ownerDocument.createTreeWalker(t,NodeFilter.SHOW_ELEMENT),i=r.currentNode;for(;i;){if(C(i))return i;i=r.nextNode()}}function T(e){var t;(t=w(e))==null||t.focus()}var E=new WeakMap;function D(e){if(!m(e,!1))return;let n=e.ownerDocument;if(!e.dispatchEvent(new t(`beforetoggle`,{cancelable:!0,oldState:`closed`,newState:`open`}))||!m(e,!1))return;let i=!1,a=e.popover,u=null,f=S(e,s.get(n)||new Set),p=S(e,c.get(n)||new Set);if(a===`auto`&&(A(c.get(n)||new Set,i,!0),M(f||n,i,!0),u=`auto`),a===`hint`&&(p?(M(p,i,!0),u=`hint`):(A(c.get(n)||new Set,i,!0),f?(M(f,i,!0),u=`auto`):u=`hint`)),a===`auto`||a===`hint`){if(a!==e.popover||!m(e,!1))return;_(n)||(i=!0),u===`auto`?(s.has(n)||s.set(n,new Set),s.get(n).add(e)):u===`hint`&&(c.has(n)||c.set(n,new Set),c.get(n).add(e))}E.delete(e);let h=n.activeElement;e.classList.add(`:popover-open`),l.set(e,`showing`),o.has(n)||o.set(n,new Set),o.get(n).add(e),I(d.get(e),!0),T(e),i&&h&&e.popover===`auto`&&E.set(e,h),r(e,`closed`,`open`)}function O(e,n=!1,i=!1){var a,u;if(!m(e,!0))return;let p=e.ownerDocument;if([`auto`,`hint`].includes(e.popover)&&(M(e,n,i),!m(e,!0)))return;let h=s.get(p)||new Set,g=h.has(e)&&f(h)===e;if(I(d.get(e),!1),d.delete(e),i&&(e.dispatchEvent(new t(`beforetoggle`,{oldState:`open`,newState:`closed`})),g&&f(h)!==e&&M(e,n,i),!m(e,!0)))return;(a=o.get(p))==null||a.delete(e),h.delete(e),(u=c.get(p))==null||u.delete(e),e.classList.remove(`:popover-open`),l.set(e,`hidden`),i&&r(e,`open`,`closed`);let _=E.get(e);_&&(E.delete(e),n&&_.focus())}function k(e,t=!1,n=!1){let r=_(e);for(;r;)O(r,t,n),r=_(e)}function A(e,t=!1,n=!1){let r=v(e);for(;r;)O(r,t,n),r=v(e)}function j(e,t,n,r){let i=!1,a=!1;for(;i||!a;){a=!0;let o=null,s=!1;for(let n of t)if(n===e)s=!0;else if(s){o=n;break}if(!o)return;for(;u(o)===`showing`&&t.size;)O(f(t),n,r);t.has(e)&&f(t)!==e&&(i=!0),i&&(r=!1)}}function M(e,t,n){let r=e.ownerDocument||e;if(e instanceof Document)return k(r,t,n);if(c.get(r)?.has(e)){j(e,c.get(r),t,n);return}A(c.get(r)||new Set,t,n),s.get(r)?.has(e)&&j(e,s.get(r),t,n)}var N=new WeakMap;function P(e){if(!e.isTrusted)return;let t=e.composedPath()[0];if(!t)return;let n=t.ownerDocument;if(!_(n))return;let r=g(t);if(r&&e.type===`pointerdown`)N.set(n,r);else if(e.type===`pointerup`){let e=N.get(n)===r;N.delete(n),e&&M(r||n,!1,!0)}}var F=new WeakMap;function I(e,t=!1){if(!e)return;F.has(e)||F.set(e,e.getAttribute(`aria-expanded`));let n=e.popoverTargetElement;if(n instanceof HTMLElement&&n.popover===`auto`)e.setAttribute(`aria-expanded`,String(t));else{let t=F.get(e);t?e.setAttribute(`aria-expanded`,t):e.removeAttribute(`aria-expanded`)}}var L=globalThis.ShadowRoot||function(){};function R(){return typeof HTMLElement<`u`&&typeof HTMLElement.prototype==`object`&&`popover`in HTMLElement.prototype}function z(e,t,n){let r=e[t];Object.defineProperty(e,t,{value(e){return r.call(this,n(e))}})}var B=/(^|[^\\]):popover-open\b/g;function V(){return typeof globalThis.CSSLayerBlockRule==`function`}function H(){let e=V();return`
${e?`@layer popover-polyfill {`:``}
  :where([popover]) {
    position: fixed;
    z-index: 2147483647;
    inset: 0;
    padding: 0.25em;
    width: fit-content;
    height: fit-content;
    border-width: initial;
    border-color: initial;
    border-image: initial;
    border-style: solid;
    background-color: canvas;
    color: canvastext;
    overflow: auto;
    margin: auto;
  }

  :where([popover]:not(.\\:popover-open)) {
    display: none;
  }

  :where(dialog[popover].\\:popover-open) {
    display: block;
  }

  :where(dialog[popover][open]) {
    display: revert;
  }

  :where([anchor].\\:popover-open) {
    inset: auto;
  }

  :where([anchor]:popover-open) {
    inset: auto;
  }

  @supports not (background-color: canvas) {
    :where([popover]) {
      background-color: white;
      color: black;
    }
  }

  @supports (width: -moz-fit-content) {
    :where([popover]) {
      width: -moz-fit-content;
      height: -moz-fit-content;
    }
  }

  @supports not (inset: 0) {
    :where([popover]) {
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
    }
  }
${e?`}`:``}
`}var U=null;function W(e){let t=H();if(U===null)try{U=new CSSStyleSheet,U.replaceSync(t)}catch{U=!1}if(U===!1){let n=document.createElement(`style`);n.textContent=t,e instanceof Document?e.head.prepend(n):e.prepend(n)}else e.adoptedStyleSheets=[U,...e.adoptedStyleSheets]}function G(){if(typeof window>`u`)return;window.ToggleEvent=window.ToggleEvent||t;function e(e){return e?.includes(`:popover-open`)&&(e=e.replace(B,`$1.\\:popover-open`)),e}z(Document.prototype,`querySelector`,e),z(Document.prototype,`querySelectorAll`,e),z(Element.prototype,`querySelector`,e),z(Element.prototype,`querySelectorAll`,e),z(Element.prototype,`matches`,e),z(Element.prototype,`closest`,e),z(DocumentFragment.prototype,`querySelectorAll`,e),Object.defineProperties(HTMLElement.prototype,{popover:{enumerable:!0,configurable:!0,get(){if(!this.hasAttribute(`popover`))return null;let e=(this.getAttribute(`popover`)||``).toLowerCase();return e===``||e==`auto`?`auto`:e==`hint`?`hint`:`manual`},set(e){e===null?this.removeAttribute(`popover`):this.setAttribute(`popover`,e)}},showPopover:{enumerable:!0,configurable:!0,value(e={}){D(this)}},hidePopover:{enumerable:!0,configurable:!0,value(){O(this,!0,!0)}},togglePopover:{enumerable:!0,configurable:!0,value(e={}){return typeof e==`boolean`&&(e={force:e}),l.get(this)===`showing`&&e.force===void 0||e.force===!1?O(this,!0,!0):(e.force===void 0||e.force===!0)&&D(this),l.get(this)===`showing`}}});let n=Element.prototype.attachShadow;n&&Object.defineProperties(Element.prototype,{attachShadow:{enumerable:!0,configurable:!0,writable:!0,value(e){let t=n.call(this,e);return W(t),t}}});let r=HTMLElement.prototype.attachInternals;r&&Object.defineProperties(HTMLElement.prototype,{attachInternals:{enumerable:!0,configurable:!0,writable:!0,value(){let e=r.call(this);return e.shadowRoot&&W(e.shadowRoot),e}}});let i=new WeakMap;function a(e){Object.defineProperties(e.prototype,{popoverTargetElement:{enumerable:!0,configurable:!0,set(e){if(e===null)this.removeAttribute(`popovertarget`),i.delete(this);else if(e instanceof Element)this.setAttribute(`popovertarget`,``),i.set(this,e);else throw TypeError(`popoverTargetElement must be an element or null`)},get(){if(this.localName!==`button`&&this.localName!==`input`||this.localName===`input`&&this.type!==`reset`&&this.type!==`image`&&this.type!==`button`||this.disabled||this.form&&this.type===`submit`)return null;let e=i.get(this);if(e&&e.isConnected)return e;if(e&&!e.isConnected)return i.delete(this),null;let t=y(this),n=this.getAttribute(`popovertarget`);return(t instanceof Document||t instanceof L)&&n&&t.getElementById(n)||null}},popoverTargetAction:{enumerable:!0,configurable:!0,get(){let e=(this.getAttribute(`popovertargetaction`)||``).toLowerCase();return e===`show`||e===`hide`?e:`toggle`},set(e){this.setAttribute(`popovertargetaction`,e)}}})}a(HTMLButtonElement),a(HTMLInputElement);let o=e=>{if(e.defaultPrevented)return;let t=e.composedPath(),n=t[0];if(!(n instanceof Element)||n?.shadowRoot)return;let r=y(n);if(!(r instanceof L||r instanceof Document))return;let i=t.find(e=>e.matches?.call(e,`[popovertargetaction],[popovertarget]`));if(i){p(i),e.preventDefault();return}},s=e=>{let t=e.key,n=e.target;!e.defaultPrevented&&n&&(t===`Escape`||t===`Esc`)&&M(n.ownerDocument,!0,!0)};(e=>{e.addEventListener(`click`,o),e.addEventListener(`keydown`,s),e.addEventListener(`pointerdown`,P),e.addEventListener(`pointerup`,P)})(document),W(document)}R()||G();