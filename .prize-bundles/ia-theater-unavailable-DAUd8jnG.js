import{A as e,C as t,E as n,N as r,O as i,P as a,T as o,w as s}from"./vendor-lit-MQ4DzmMc.js";import{n as c}from"./vendor-topnav-Dcg-INpV.js";import{t as l}from"./index-NFRVgqV2.js";function u(e){return encodeURIComponent(e).replace(/[!'()*]/g,e=>`%${e.charCodeAt(0).toString(16).toUpperCase()}`)}function d(e){return e.split(`/`).map(u).join(`/`)}var f=class extends o{constructor(...e){super(...e),this.variant=`no-preview`,this.baseHost=`https://archive.org`}render(){let e;switch(this.variant){case`login-required`:e=this.loginRequiredContent;break;case`load-failed`:e=this.loadFailedContent;break;default:e=this.noPreviewContent;break}return i` <div class="content">${e}</div> `}get noPreviewContent(){return i`
      <p class="title">${a(`There Is No Preview Available For This Item`)}</p>
      <div class="subtitle">
        <p>
          ${a(`This item does not appear to have any files that can be experienced on Archive.org.`)}
        </p>
        <p>
          ${a(`Please download files in this item to interact with them on your computer.`)}
        </p>
      </div>
      ${this.identifier?i`<a
            href=${c(this.identifier,``,this.baseHost)}
            class="action-btn"
          >
            ${a(`Show all files`)}
          </a>`:n}
    `}get loadFailedContent(){return i`
      <p class="title">${a(`The Player Could Not Be Loaded`)}</p>
      <div class="subtitle">
        <p>
          ${a(`Please download files in this item to interact with them on your computer.`)}
        </p>
      </div>
      ${this.identifier?i`<a
            href=${c(this.identifier,``,this.baseHost)}
            class="action-btn"
          >
            ${a(`Show all files`)}
          </a>`:n}
    `}get loginRequiredContent(){return i`
      <svg
        class="person-icon"
        xmlns="http://www.w3.org/2000/svg"
        viewBox="0 0 24 24"
        aria-hidden="true"
      >
        <path
          d="M12 12c2.7 0 4.8-2.1 4.8-4.8S14.7 2.4 12 2.4 7.2 4.5 7.2 7.2 9.3 12 12 12zm0 2.4c-3.2 0-9.6 1.6-9.6 4.8v2.4h19.2v-2.4c0-3.2-6.4-4.8-9.6-4.8z"
        />
      </svg>
      <p class="title">${a(`You must log in to view this content`)}</p>
      <a href="${this.baseHost}/login" class="action-btn"> ${a(`Log in`)} </a>
      <p class="login-state">
        ${a(`Don't have an account?`)}
        <a href="${this.baseHost}/signup">${a(`Sign up for free!`)}</a>
      </p>
    `}static get styles(){return e`
      :host {
        display: flex;
        width: 100%;
        height: 100%;
        align-items: center;
        justify-content: center;
      }

      .content {
        display: flex;
        flex-direction: column;
        align-items: center;
        text-align: center;
        color: #fff;
        padding: 2rem;
      }

      .title {
        font-size: 2.6rem;
        font-weight: bold;
        text-transform: uppercase;
        margin: 0;
      }

      .subtitle {
        font-size: 2rem;
        margin: 2rem 0 0 0;
        opacity: 0.8;
      }
      .subtitle p {
        margin: 0;
      }

      .action-btn {
        display: inline-block;
        margin-top: 1.75rem;
        padding: 1rem 2rem;
        background: #4a90e2;
        color: #fff;
        font-size: 1.8rem;
        border-radius: 4px;
        text-decoration: none;
      }
      .action-btn:hover {
        background: #fff;
        color: #000;
      }

      .person-icon {
        width: 6rem;
        height: 6rem;
        fill: #fff;
        margin-bottom: 2rem;
      }

      .login-state {
        font-size: 1.4rem;
        opacity: 0.7;
        margin: 2rem 0;
      }
      .login-state a {
        color: #fff;
      }
    `}};l([t({type:String})],f.prototype,`variant`,void 0),l([t({type:String})],f.prototype,`baseHost`,void 0),l([t({type:String})],f.prototype,`identifier`,void 0),f=l([s(`ia-theater-unavailable`),r()],f);export{d as t};