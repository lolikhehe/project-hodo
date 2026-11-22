import{A as e,C as t,O as n,S as r,T as i,k as a,w as o}from"./vendor-lit-MQ4DzmMc.js";import{o as s}from"./vendor-shared-62wh2Xkx.js";var c=a`
  <svg
    class="icon-loader"
    height="20"
    viewBox="0 0 128 128"
    width="20"
    xmlns="http://www.w3.org/2000/svg"
  >
    <g>
      <circle cx="16" cy="64" r="16" class="fill-color" fill-opacity="1" />
      <circle cx="16" cy="64" r="14.344" class="fill-color" fill-opacity="1" transform="rotate(45 64 64)" />
      <circle cx="16" cy="64" r="12.531" class="fill-color" fill-opacity="1" transform="rotate(90 64 64)" />
      <circle cx="16" cy="64" r="10.75" class="fill-color" fill-opacity="1" transform="rotate(135 64 64)" />
      <circle cx="16" cy="64" r="10.063" class="fill-color" fill-opacity="1" transform="rotate(180 64 64)" />
      <circle cx="16" cy="64" r="8.063" class="fill-color" fill-opacity="1" transform="rotate(225 64 64)" />
      <circle cx="16" cy="64" r="6.438" class="fill-color" fill-opacity="1" transform="rotate(270 64 64)" />
      <circle cx="16" cy="64" r="5.375" class="fill-color" fill-opacity="1" transform="rotate(315 64 64)" />
      <animateTransform
        attributeName="transform"
        type="rotate"
        values="0 64 64;315 64 64;270 64 64;225 64 64;180 64 64;135 64 64;90 64 64;45 64 64"
        calcMode="discrete"
        dur="720ms"
        repeatCount="indefinite"
      />
    </g>
  </svg>
`,l=a`
  <svg
    class="icon-question"
    height="20"
    viewBox="0 0 20 20"
    width="20"
    xmlns="http://www.w3.org/2000/svg"
  >
    <g class="fill-color" fill-rule="nonzero">
      <path
        d="m10.7078125 12.8359375v-.4140625c0-.765625.2890625-1.1484375 1.328125-1.75
           1.1015625-.6484375 1.671875-1.46875 1.671875-2.6484375
           0-1.78125-1.4765625-3.0234375-3.7109375-3.0234375
           -2.3984375 0-3.7578125 1.34375-3.796875 3.265625h2.1171875
           c.046875-.8828125.640625-1.453125 1.5390625-1.453125
           .890625 0 1.484375.5390625 1.484375 1.28125s-.3046875 1.125-1.3125 1.7265625
           c-1.078125.6328125-1.5078125 1.3359375-1.40625 2.5625l.015625.453125z"
      />
      <path
        d="m9.7 16.57c.711 0 1.281-.539 1.281-1.227s-.57-1.226-1.28-1.226
           c-.704 0-1.274.539-1.274 1.226s.57 1.227 1.274 1.227z"
      />
      <path
        d="m10 0c-5.523 0-10 4.477-10 10s4.477 10 10 10 10-4.477 10-10-4.477-10-10-10z
           m0 2c4.418 0 8 3.582 8 8s-3.582 8-8 8-8-3.582-8-8 3.582-8 8-8z"
      />
    </g>
  </svg>
`,u=()=>({events:{},emit(e,...t){(this.events[e]||[]).forEach(e=>e(...t))},on(e,t){return(this.events[e]=this.events[e]||[]).push(t),()=>this.events[e]=(this.events[e]||[]).filter(e=>e!==t)}});function d(e){return new Promise(t=>setTimeout(t,e))}var f;(function(e){e.retryNumber=`retryNumber`,e.owner=`owner`,e.dynamicImportLoaded=`dynamicImportLoaded`,e.hasBeenRetried=`hasBeenRetried`})(f||(f={}));var p=`lazyLoaderService`,m=new class{constructor(e){this.emitter=u(),this.container=e?.container??document.head,this.retryCount=e?.retryCount??2,this.retryInterval=e?.retryInterval??1}on(e,t){return this.emitter.on(e,t)}loadBundle(e){return s(this,void 0,void 0,function*(){let t,n;return e.module&&(t=this.loadScript({src:e.module,bundleType:`module`})),e.nomodule&&(n=this.loadScript({src:e.nomodule,bundleType:`nomodule`})),Promise.race([t,n])})}loadScript(e){return s(this,void 0,void 0,function*(){return this.doLoad(e)})}doLoad(e){return s(this,void 0,void 0,function*(){let t=e.retryNumber??0,n=`script[src='${e.src}'][async][${f.owner}='${p}'][${f.retryNumber}='${t}']`,r=this.container.querySelector(n);return r||(r=this.getScriptTag(Object.assign(Object.assign({},e),{retryNumber:t})),this.container.appendChild(r)),new Promise((n,i)=>{if(r.getAttribute(f.dynamicImportLoaded)){n();return}let a=e.scriptBeingRetried,o=r.onload||a?.onload;r.onload=e=>{o?.(e),r.setAttribute(f.dynamicImportLoaded,`true`),n()};let c=r.onerror||a?.onerror;r.onerror=n=>s(this,void 0,void 0,function*(){let a=r.getAttribute(f.hasBeenRetried);if(t<this.retryCount&&!a){r.setAttribute(f.hasBeenRetried,`true`),yield d(this.retryInterval*1e3);let n=t+1;this.emitter.emit(`scriptLoadRetried`,e.src,n),this.doLoad(Object.assign(Object.assign({},e),{retryNumber:n,scriptBeingRetried:r}))}else a||this.emitter.emit(`scriptLoadFailed`,e.src,n),c?.(n),i(n)})})})}getScriptTag(e){let t=e.src.replace(`'`,`"`),n=document.createElement(`script`),r=e.retryNumber;n.setAttribute(f.owner,p),n.setAttribute(`src`,t),n.setAttribute(f.retryNumber,r.toString()),n.async=!0;let i=e.attributes??{};switch(Object.keys(i).forEach(e=>{n.setAttribute(e,i[e])}),e.bundleType){case`module`:n.setAttribute(`type`,e.bundleType);break;case`nomodule`:n.setAttribute(e.bundleType,``);break}return n}},h=`https://static.zdassets.com/ekr/snippet.js`;function g(e){return m.loadScript({src:`${h}?key=${e}`,attributes:{id:`ze-snippet`}})}function _(e=1e4){return new Promise((t,n)=>{let r=setInterval(()=>{window.zE&&(clearInterval(r),clearTimeout(i),t())},100),i=setTimeout(()=>{clearInterval(r),n(Error(`Zendesk API did not initialise in time`))},e)})}var v=Object.defineProperty,y=Object.getOwnPropertyDescriptor,b=(e,t,n,r)=>{for(var i=r>1?void 0:r?y(t,n):t,a=e.length-1,o;a>=0;a--)(o=e[a])&&(i=(r?o(t,n,i):o(i))||i);return r&&i&&v(t,n,i),i},x=class extends i{constructor(){super(...arguments),this.isLoading=!1,this.buttonVisible=!0,this.zendeskReady=!1}async initiateZenDesk(){this.isLoading=!0,this.dispatchEvent(new Event(`zendeskHelpButtonClicked`));try{if(!this.widgetKey)throw Error(`Missing widgetKey`);if(!this.zendeskReady){if(await g(this.widgetKey),await _(),!window.zE){this.isLoading=!1;return}window.zE(`messenger:on`,`open`,()=>{this.buttonVisible=!1,this.isLoading=!1}),window.zE(`messenger:on`,`close`,()=>{setTimeout(()=>{this.buttonVisible=!0},500)}),this.zendeskReady=!0}window.zE&&window.zE(`messenger`,`open`)}catch(e){this.isLoading=!1,console.error(`[ia-zendesk-help-widget]`,e)}}get iconTemplate(){return this.isLoading?c:l}render(){return n`
      <button
        class="help-widget ${this.buttonVisible?``:`hidden`}"
        @click=${this.initiateZenDesk}
      >
        ${this.iconTemplate}
        <span class="label">Help</span>
      </button>
    `}static get styles(){return e`
      .help-widget {
        position: fixed;
        top: var(--button-top, auto);
        bottom: var(--button-bottom, 0);
        left: var(--button-left, auto);
        right: var(--button-right, 0);
        z-index: var(--button-z-index, 999998);
        width: var(--button-width, auto);
        padding: var(--button-padding, 14px);
        margin: var(--button-margin, 14px 20px);
        background: var(--button-background, #194880);
        color: var(--button-color, #fff);
        border-radius: var(--button-border-radius, 999rem);
        border: 0;
        font-size: var(--button-font-size, 14px);
        font-weight: var(--button-font-weight, 700);
        letter-spacing: 0.6px;
        outline: none;
        cursor: pointer;
        vertical-align: middle;
        transition: opacity 0.12s linear;
      }

      .fill-color {
        fill: var(--button-color, #fff);
      }

      .help-widget svg {
        vertical-align: middle;
        pointer-events: none;
      }

      .label {
        pointer-events: none;
        margin-left: 5px;
      }

      .hidden {
        opacity: 0;
        display: none;
        visibility: hidden;
      }

      @media (max-width: 767px) {
        .label {
          display: none;
        }
      }
    `}};b([t({type:String})],x.prototype,`widgetKey`,2),b([r()],x.prototype,`isLoading`,2),b([r()],x.prototype,`buttonVisible`,2),x=b([o(`ia-zendesk-help-widget`)],x);export{x as IAZendeskHelpWidget};