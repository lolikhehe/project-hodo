import{A as e,C as t,E as n,O as r,S as i,T as a,b as o,w as s,x as c}from"./vendor-lit-MQ4DzmMc.js";import{t as l}from"./index-NFRVgqV2.js";import{t as u}from"./purify.es-RmFZhtLI.js";import"./ia-icon-ClrHaxRi.js";import{t as d}from"./edit-BYjEX3eL.js";import"./ia-forum-jKG4SJKA.js";var f=`a.abbr.b.blockquote.br.cite.code.div.em.h1.h2.h3.h4.h5.h6.hr.i.img.li.ol.p.q.section.span.strong.style.sub.sup.table.tbody.td.th.thead.tr.ul.var`.split(`.`),p=[`align`,`alt`,`border`,`class`,`height`,`href`,`src`,`style`,`title`,`width`],m=class extends a{render(){return this.newsStories?r`<section class="news">
      <h2 class="news-title">
        News <a class="more-news-link" href="/about/news-stories"> [more]</a>
      </h2>
      <table class="news-stories">
        <tbody>
          ${this.newsStories.map(e=>r`<tr>
                <td class="news-item">
                  <a href=${e.url} target="_blank" class="news-link"
                    >${e.headline}</a
                  >
                </td>
              </tr>`)}
        </tbody>
      </table>
    </section>`:n}updated(e){e.has(`metadataService`)&&this.metadataService&&this.fetchNewsStories()}async fetchNewsStories(){let e=await this.localCache?.get(`ia-news-stories`);if(e)this.newsStories=e;else{let e=await this.metadataService?.fetchMetadataValue(`press_mentions`,`topten/mentions`);if(e?.success){let t=e.success;this.newsStories=t,await this.localCache?.set({key:`ia-news-stories`,value:t,ttl:300})}}}static get styles(){return e`
      .news-title {
        font-size: 2.4rem;
        font-weight: 500;
        margin-bottom: 10px;
        line-height: 1.1;
        margin-top: 0;
        display: inline-block;
      }

      .more-news-link {
        display: inline-block;
        font-weight: 500;
        font-size: 1rem;
      }

      .news-stories {
        border-collapse: collapse;
      }

      .news-stories > tbody {
        vertical-align: middle;
      }

      .news-item {
        border-top: 1px solid #ddd;
        padding: 5px;
        font-size: 1.4rem;
      }

      .news-item:hover,
      .news-item:active,
      .news-item:focus {
        background-color: var(--ia-theme-primary-background-color, #fbfbfd);
      }

      .news-link:link,
      .news-link:visited {
        color: var(--ia-theme-link-color, #4b64ff);
        text-decoration: none;
      }

      .news-link:hover {
        text-decoration: underline;
      }
    `}};l([t({type:Object})],m.prototype,`metadataService`,void 0),l([t({type:Object})],m.prototype,`localCache`,void 0),l([i()],m.prototype,`newsStories`,void 0),m=l([s(`news-stories`)],m);var h=class extends a{constructor(...e){super(...e),this.scriptURL=`https://workforcenow.adp.com/mascsr/default/mdf/recwebcomponents/recruitment/main-config/recruitment.js`,this.contentLoaded=!1,this.scriptError=!1}render(){return this.scriptError?r`<p class="error-msg">
        There was a problem loading the job listings. Please try again later.
      </p>`:r`${this.contentLoaded?n:r`<div class="placeholder">
            <div class="placeholder-tile"></div>
            <div class="placeholder-tile"></div>
            <div class="placeholder-tile"></div>
            <div class="placeholder-tile"></div>
            <div class="placeholder-tile"></div>
            <div class="placeholder-tile"></div>
          </div>`}

      <recruitment-current-openings
        cid="12261ced-819c-4d6d-af85-ffea690b6521"
        ccid="9201102651278_3"
        host="DP"
        locale="en_US"
      ></recruitment-current-openings>`}updated(e){e.has(`lazyLoaderService`)&&this.lazyLoaderService&&this.lazyLoaderService?.loadScript({src:this.scriptURL}).then(()=>this.checkContentLoaded()).catch(()=>this.scriptError=!0)}checkContentLoaded(){if(!this.currentOpenings)return;let e=new MutationObserver(()=>{this.contentLoaded=!0,e.disconnect()});e.observe(this.currentOpenings,{attributes:!0})}static get styles(){return e`
      .placeholder-tile {
        width: 100%;
        height: 90px;
        background: linear-gradient(to right bottom, #dddddd, #d8d8d8);
        border-radius: 7.5px;
        margin: 7.5px 0;
      }
    `}};l([t({type:Object})],h.prototype,`lazyLoaderService`,void 0),l([t({type:String})],h.prototype,`scriptURL`,void 0),l([i()],h.prototype,`contentLoaded`,void 0),l([i()],h.prototype,`scriptError`,void 0),l([c(`recruitment-current-openings`)],h.prototype,`currentOpenings`,void 0),h=l([s(`jobs-widget`)],h);var g=class extends a{constructor(...e){super(...e),this.page=`about`,this.canEdit=!1}firstUpdated(){this.setupAppServices()}updated(e){e.has(`metadataService`)&&this.metadataService&&this.fetchContent(),e.has(`userService`)&&this.userService&&this.setPermissions(),e.has(`pageContent`)&&(this.renderContent(),this.setupNavigation(),this.setupPageMetadata())}render(){return r`<section class="static-page">
      ${this.canEdit?this.editButtonTemplate:n}
      <div class="content-wrap">
        ${this.page===`terms`?n:this.sideMatterTemplate}
        <section class="content ${this.page}">
          <div id="content-container"></div>
          ${this.page===`jobs`?r`<jobs-widget
                .lazyLoaderService=${this.lazyLoaderService}
              ></jobs-widget>`:n}
        </section>
      </div>
      ${this.page===`sflan`||this.page===`petabox`?r`<ia-forum
            .identifier=${this.page}
            .forumService=${this.forumService}
          ></ia-forum>`:n}
    </section>`}get editButtonTemplate(){return r`<a href=${`/editxml/ia-page-${this.page}`} class="edit-button">
      <ia-icon .src=${d}></ia-icon>
      <span class="edit-label">Edit</span>
    </a>`}get sideMatterTemplate(){let e=this.pageContent?.sidebarLinks;return r`<div class="sidebar">
      ${e?r`<section class="sidebar-links">
            <h3 class="sidebar-section-title">${e.heading}</h3>
            ${e.links.map(e=>r`<a class="sidebar-link" href=${e.url}
                  >${e.title}</a
                >`)}
          </section>`:n}
      <news-stories
        .metadataService=${this.metadataService}
        .localCache=${this.localCache}
      ></news-stories>
    </div>`}renderContent(){if(!this.contentContainer)return;let e=this.sanitize(this.pageContent?.mainContent??``);this.contentContainer.innerHTML=e}setupNavigation(){this.navigators&&this.navigators.forEach(e=>e.addEventListener(`click`,e=>{let t=e.target.innerText;if(this.locators){let e=Array.from(this.locators).filter(e=>e.innerText===t)[0];e&&(e?.scrollIntoView(),e?.focus())}}))}async fetchContent(){let e=`ia-page-${this.page}`,t={mainContent:``},n=await this.localCache?.get(e);if(n)t=n;else{let n=await this.metadataService?.fetchMetadata(e);if(n?.success){let r=n.success.metadata;t={title:r.title?.value??``,mainContent:r.description?.value??``};let i=r.rawMetadata?.sidebar_links;i&&(t.sidebarLinks=JSON.parse(i)),await this.localCache?.set({key:e,value:t,ttl:900})}}this.pageContent=t}async setupAppServices(){let e=await this.appServices?.get();await Promise.all([e?.metadataService.get().then(e=>{this.metadataService=e}),e?.localCache.get().then(e=>{this.localCache=e}),e?.lazyLoaderService.get().then(e=>{this.lazyLoaderService=e}),e?.pageMetadataService.get().then(e=>{this.pageMetadataService=e}),e?.userService.get().then(e=>{this.userService=e}),e?.forumService.get().then(e=>{this.forumService=e})])}async setupPageMetadata(){let e=this.pageContent?.title;e&&this.pageMetadataService&&this.pageMetadataService?.setTitle(e)}async setPermissions(){let e=(await this.userService?.getLoggedInUser())?.success;e&&(this.canEdit=e.privs.includes(`/`)||e.privs.includes(`/admins`))}sanitize(e){return u.sanitize(e,{ALLOWED_TAGS:f,ALLOWED_ATTR:p})}static get styles(){return e`
      .static-page {
        background-color: var(--ia-theme-secondary-background-color, white);
        min-height: 100vh;
      }

      .content-wrap {
        display: flex;
        flex-direction: row;
        max-width: 117rem;
        margin: 0 auto;
        padding: 20px 0px;
      }

      .content {
        font-size: 1.4rem;
        padding: 0 15px;
      }

      a:link.edit-button,
      a:visited.edit-button {
        position: absolute;
        right: 10px;
        padding-top: 10px;
        display: flex;
        flex-direction: row;
        align-items: center;
        gap: 3px;
        color: var(--actionButtonNormalForegroundColor, #767676);
      }

      a:link.edit-button:hover,
      a:visited.edit-button:hover {
        text-decoration: none;
        color: var(--ia-theme-link-color, #4b64ff);
      }

      ia-icon {
        --iconWidth: 1.4rem;
        --iconHeight: 1.4rem;
      }

      .edit-label {
        font-size: 1.4rem;
        color: inherit;
        text-decoration: none;
      }

      .sidebar {
        width: calc(25% - 30px);
        padding: 10px 15px;
        flex: none;
      }

      .sidebar-links {
        font-size: 1.4rem;
        margin-bottom: 30px;
      }

      .sidebar-section-title {
        margin-top: 0;
      }

      .sidebar-link {
        display: block;
        margin-bottom: 10px;
      }

      /* Generic styles */

      h1,
      h2,
      h3 {
        font-weight: 500;
        margin-bottom: 10px;
        line-height: 1.1;
      }

      h1 {
        margin-top: 0px;
      }

      h2,
      h3 {
        margin-top: 20px;
      }

      h1 {
        font-size: 3.6rem;
      }

      h2 {
        font-size: 3rem;
      }

      h3 {
        font-size: 2.4rem;
      }

      a:link,
      a:visited,
      .navigator {
        color: var(--ia-theme-link-color, #4b64ff);
        text-decoration: none;
      }

      a:hover,
      .navigator:hover {
        cursor: pointer;
        text-decoration: underline;
      }

      p {
        margin: 0 0 10px;
      }

      hr {
        margin: 15px 0 10px 0;
        border-top: 1px solid var(--ia-theme-primary-background-color, #fbfbfd);
      }

      table {
        border-collapse: collapse;
        margin: 0 auto;
      }

      tbody {
        vertical-align: top;
      }

      .centered {
        margin-left: auto;
        margin-right: auto;
        text-align: center;
      }

      ia-forum {
        max-width: 117rem;
        padding: 0 15px;
        margin: 0 20px;
      }

      /* Subpage-specific styles */

      .content.about .partner-logo {
        width: 80px;
        padding: 10px;
        height: 80px;
        object-fit: contain;
      }

      .content.about .internet-archive-photo {
        margin-left: 20px;
        float: right;
        max-width: 100%;
        width: 300px;
        height: 200px;
      }

      .content.bios h3 {
        margin-bottom: 20px;
      }

      .content.bios img {
        border-radius: 7px;
      }

      .content.contact h3 {
        font-size: 14px;
        font-weight: 700;
        margin-top: 0;
        margin-bottom: 10px;
        padding-top: 5px;
      }

      .content.contact img[alt='Open Street Map preview'] {
        max-width: 100%;
      }

      .content.credits table {
        margin: 0 auto;
      }

      .content.credits tbody {
        vertical-align: middle;
      }

      .content.credits td {
        border-bottom: 1px solid #cccc99;
        padding: 10px;
        text-align: right;
      }

      .content.credits img[alt='Library of Congress Logo'] {
        width: 100px;
      }

      .content.sflan .sflan-image {
        float: right;
        padding: 10px;
      }

      .content.petabox .contributors {
        margin: 0;
      }

      .content.petabox .contributors td {
        padding-right: 15px;
      }

      .content.projects td {
        padding: 15px;
      }

      @media (max-width: 420px) {
        .content.about .internet-archive-photo {
          margin-left: 0;
          float: none;
        }
      }

      @media only screen and (max-width: 992px) {
        .sidebar {
          display: none;
        }

        .content.projects tr {
          display: flex;
          flex-direction: column;
          padding-bottom: 10px;
          margin-bottom: 20px;
          border-bottom: 1px solid
            var(--ia-theme-primary-background-color, #fbfbfd);
        }

        .content.projects td {
          padding: 0;
        }
      }
    `}};l([t({type:String})],g.prototype,`page`,void 0),l([i()],g.prototype,`pageContent`,void 0),l([t({type:Object})],g.prototype,`appServices`,void 0),l([i()],g.prototype,`metadataService`,void 0),l([i()],g.prototype,`lazyLoaderService`,void 0),l([i()],g.prototype,`localCache`,void 0),l([i()],g.prototype,`pageMetadataService`,void 0),l([i()],g.prototype,`userService`,void 0),l([i()],g.prototype,`forumService`,void 0),l([i()],g.prototype,`canEdit`,void 0),l([c(`#content-container`)],g.prototype,`contentContainer`,void 0),l([o(`.navigator`)],g.prototype,`navigators`,void 0),l([o(`.locator`)],g.prototype,`locators`,void 0),g=l([s(`static-page`)],g);var _=g;export{_ as default};