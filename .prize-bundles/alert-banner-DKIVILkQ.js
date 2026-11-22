import{A as e,C as t,E as n,O as r,T as i,w as a}from"./vendor-lit-MQ4DzmMc.js";import{t as o}from"./index-NFRVgqV2.js";var s=`data:image/svg+xml,%3csvg%20xmlns='http://www.w3.org/2000/svg'%20width='16'%20height='16'%20viewBox='0%200%2016%2016'%3e%3cg%20fill='none'%20fill-rule='evenodd'%3e%3ccircle%20cx='8'%20cy='8'%20r='8'%20fill='%23FFF'/%3e%3cpath%20stroke='%23000'%20stroke-linecap='round'%20d='M4.5,4.5%20L11.4121686,11.4121686'/%3e%3cpath%20stroke='%23000'%20stroke-linecap='round'%20d='M4.5,4.5%20L11.4121686,11.4121686'%20transform='matrix(-1%200%200%201%2016%200)'/%3e%3c/g%3e%3c/svg%3e`,c=`data:image/svg+xml,%3csvg%20xmlns='http://www.w3.org/2000/svg'%20width='16'%20height='16'%20viewBox='0%200%2016%2016'%3e%3cg%20fill='none'%20fill-rule='evenodd'%3e%3ccircle%20cx='8'%20cy='8'%20r='8'%20fill='%23333'/%3e%3cpath%20stroke='%23ccc'%20stroke-linecap='round'%20d='M4.5,4.5%20L11.4121686,11.4121686'/%3e%3cpath%20stroke='%23ccc'%20stroke-linecap='round'%20d='M4.5,4.5%20L11.4121686,11.4121686'%20transform='matrix(-1%200%200%201%2016%200)'/%3e%3c/g%3e%3c/svg%3e`,l=class extends i{constructor(...e){super(...e),this.level=`info`,this.showCloseButton=!1}render(){return r`
      <div class="banner-wrapper ${this.level}">
        ${this.showCloseButton?this.dismissButtonTemplate:n}
        <slot></slot>
      </div>
    `}get dismissButtonTemplate(){return r`
      <button id="close-button" @click=${this.closeButtonPressed}>
        <img src=${this.level===`warn`?c:s} />
        ${this.closeButtonText?r` <span id="dismiss-text">${this.closeButtonText}</span> `:n}
      </button>
    `}closeButtonPressed(){let e=new Event(`closeButtonPressed`);this.dispatchEvent(e)}static get styles(){return e`
      :host {
        display: block;
      }

      .banner-wrapper {
        font-size: 1.4rem;
        padding: 0.75rem 1rem;
      }

      ::slotted(a:hover) {
        text-decoration: none;
      }

      .urgent {
        background-color: #ab4517;
        color: #fff;
        border: 1px solid #67290e;
        border-top: 0;
      }

      .urgent ::slotted(*) {
        color: #fff;
      }

      .warn {
        background-color: #fae366;
        color: #2c2c2c;
        border: 1px solid #96883d;
        border-top: 0;
      }

      .warn ::slotted(*) {
        color: #2c2c2c;
      }

      .info {
        background-color: #428bca;
        color: #fff;
        border: 1px solid #285379;
        border-top: 0;
      }

      .info ::slotted(*) {
        color: #fff;
      }

      #close-button {
        cursor: pointer;
        color: inherit;
        background: none;
        border: none;
        display: flex;
        align-items: center;
        padding-right: 0;
        float: right;
      }

      #dismiss-text {
        margin-left: 0.5rem;
      }
    `}};o([t({type:String})],l.prototype,`level`,void 0),o([t({type:Boolean})],l.prototype,`showCloseButton`,void 0),o([t({type:String})],l.prototype,`closeButtonText`,void 0),l=o([a(`alert-banner`)],l);export{l as AlertBanner};