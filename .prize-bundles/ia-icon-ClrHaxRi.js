import{A as e,C as t,E as n,O as r,T as i,w as a}from"./vendor-lit-MQ4DzmMc.js";import{t as o}from"./index-NFRVgqV2.js";var s=class extends i{constructor(...e){super(...e),this.src=``}render(){return this.src?r`
      <div class="icon masked" style="mask-image: url('${this.src.replace(/'/g,`%27`).replace(/"/g,`%22`)}')"></div>
      <img class="icon fallback" src=${this.src} />
    `:n}static get styles(){return e`
      .icon {
        height: var(--iconHeight, 2rem);
        width: var(--iconWidth, var(--iconHeight, 2rem));
        filter: var(--iconFilter, none);
      }

      .icon.masked {
        display: none;
      }

      /* Icons inherit font color when possible */
      @supports (mask-image: url()) and (background: currentColor) {
        .icon.masked {
          display: block;
          mask-repeat: no-repeat;
          mask-size: 100%;
          background: var(--iconColor, currentColor);
          filter: none;
          transition: var(--iconTransition, none);
        }

        .icon.fallback {
          display: none;
        }
      }
    `}};o([t({type:String})],s.prototype,`src`,void 0),s=o([a(`ia-icon`)],s);