import{A as e,C as t,E as n,O as r,S as i,T as a,j as o,l as s,w as c}from"./vendor-lit-MQ4DzmMc.js";import{t as l}from"./index-NFRVgqV2.js";import{t as u}from"./email-link-DbmrSSqK.js";import{t as d}from"./rss-ogHre5Wb.js";var f=class extends a{constructor(...e){super(...e),this.forum=n}render(){return this.identifier?r`<div id="forum-container">${this.forum}</div>`:n}updated(e){e.has(`forumService`)&&this.forumService&&this.updateForumPosts()}async updateForumPosts(){try{let e=this.identifier;if(!e)return;let t=await this.forumService?.fetchForumHTML(e);if(t?.success){let e=t.success;this.forum=r`${s(e)}`;let n=new CustomEvent(`forumLoaded`,{detail:!0});this.dispatchEvent(n);return}}catch{}this.forum=n}static get styles(){return e`
      :host {
        font-size: 1.4rem;
      }

      #forum-container {
        max-width: inherit;
        margin: 0 auto;
      }

      #forum-container .box {
        padding: 10px;
        background: var(--ia-theme-secondary-background-color, white);
      }

      #forum-container h1 {
        margin-top: 0;
        font-size: 2.4rem;
        font-weight: 500;
      }

      #forum-container .label {
        display: inline;
        padding: 0.2rem 0.6rem 0.3rem;
        font-size: 75%;
        font-weight: bold;
        line-height: 1;
        color: #fff;
        text-align: center;
        white-space: nowrap;
        vertical-align: baseline;
        border-radius: 4px;
      }

      #forum-container .label-success {
        background-color: #23765d;
        text-decoration: none;
      }

      #forum-container .label-success:hover {
        background-color: #39846e;
      }

      #forum-container span.iconochive-rss {
        display: inline-flex;
      }

      #forum-container span.iconochive-rss::after {
        width: 1.4rem;
        height: 1.4rem;
        filter: invert(1); /* Icon is black; make it white */
        /* Don't let prettier convert the following quotes from " to '. They MUST be double-quotes. */
        /* prettier-ignore */
        content: url("${o(d)}");
      }

      #forum-container span.iconochive-email::after {
        width: 3rem;
        display: inline-block;
        vertical-align: text-top;
        /* Don't let prettier convert the following quotes from " to '. They MUST be double-quotes. */
        /* prettier-ignore */
        content: url("${o(u)}");
      }

      #forum-container .forumTable {
        width: 100%;
        background: transparent;
        border-collapse: collapse;
        border-spacing: 0;
        word-break: break-word;
      }

      #forum-container .forumRowHead {
        word-break: normal;
      }

      #forum-container tr:hover {
        background: var(--ia-theme-primary-background-color, #fbfbfd);
      }

      #forum-container th,
      #forum-container td {
        text-align: left;
        padding: 5px;
        line-height: 1.42857143;
        vertical-align: top;
        border-top: 1px solid #ddd;
      }

      #forum-container .smalldate {
        display: none;
      }

      .sr-only {
        display: none;
      }

      a:link {
        color: var(--ia-theme-link-color, #4b64ff);
        text-decoration: none;
      }

      a:link:hover {
        text-decoration: underline;
      }
    `}};l([t({type:String})],f.prototype,`identifier`,void 0),l([t({type:Object})],f.prototype,`forumService`,void 0),l([i()],f.prototype,`forum`,void 0),f=l([c(`ia-forum`)],f);