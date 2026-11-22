import{A as e,C as t,E as n,N as r,O as i,P as a,S as o,T as s,w as c}from"./vendor-lit-MQ4DzmMc.js";import{t as l}from"./index-NFRVgqV2.js";import{i as u}from"./dist-BqHmxVdQ.js";import"./ia-dropdown-DbuqBiPJ.js";import"./ia-icon-ClrHaxRi.js";import{a as d,i as f,o as p,r as m,s as h}from"./share-modal-content-BOnwy04n.js";import{t as g}from"./rss-ogHre5Wb.js";import{t as _}from"./edit-BYjEX3eL.js";import{t as v}from"./show-login-modal-C9u5aoLY.js";var y=`data:image/svg+xml,%3csvg%20viewBox='0%200%20100%20100'%20xmlns='http://www.w3.org/2000/svg'%3e%3cpath%20d='m100%2050.1132564-100%2049.8867436v-100z'/%3e%3c/svg%3e`,b=`data:image/svg+xml,%3csvg%20viewBox='0%200%20100%20100'%20xmlns='http://www.w3.org/2000/svg'%3e%3cpath%20d='m50%200c27.6142375%200%2050%2022.3857625%2050%2050s-22.3857625%2050-50%2050-50-22.3857625-50-50%2022.3857625-50%2050-50zm23.8159475%2026.1840525c-1.4033215-1.4033215-3.5816761-1.5592461-5.1572272-.4677738l-.5598841.4677738-18.0988362%2018.0989475-18.0988362-18.0989475-.5598841-.4677738c-1.5755511-1.0914723-3.7539057-.9355477-5.1572272.4677738-1.5787367%201.5787367-1.5787367%204.1383746%200%205.7171113l18.0989475%2018.0988362-18.0989475%2018.0988362c-1.5787367%201.5787367-1.5787367%204.1383746%200%205.7171113%201.4033215%201.4033215%203.5816761%201.5592461%205.1572272.4677738l.5598841-.4677738%2018.0988362-18.0989475%2018.0988362%2018.0989475.5598841.4677738c1.5755511%201.0914723%203.7539057.9355477%205.1572272-.4677738%201.5787367-1.5787367%201.5787367-4.1383746%200-5.7171113l-18.0989475-18.0988362%2018.0989475-18.0988362c1.5787367-1.5787367%201.5787367-4.1383746%200-5.7171113z'%20fill-rule='evenodd'/%3e%3c/svg%3e`,x=5e3,S=class extends s{constructor(...e){super(...e),this.mediaType=`search`,this.title=``,this.loggedIn=!1,this.showShare=!0,this.showFavoriteButton=!0,this.showRssButton=!1,this.showPlayAllButton=!1,this.showEditButton=!1,this.showEditxmlButton=!1,this.showRemoveItemsButton=!1,this.showHistoryButton=!1,this.showManageButton=!1,this.showSettingsButton=!1,this.manageActionText=`Manage`,this.hasFavorite=!1,this.favoriteError=!1}disconnectedCallback(){this.clearFavoriteError(),super.disconnectedCallback()}willUpdate(e){(e.has(`identifier`)||e.has(`mediaType`))&&this.clearFavoriteError()}updated(e){(e.has(`identifier`)||e.has(`mediaType`))&&this.checkFavoriteStatus()}async checkFavoriteStatus(){if(!this.identifier){this.hasFavorite=!1;return}let e=await(await this.appServices?.get())?.favoriteService?.get();try{let t=await e?.checkFavorite({identifier:this.identifier,...this.mediaType?{mediatype:this.mediaType}:{}});t?.value!=null&&(this.hasFavorite=t.value)}catch(e){console.log(e)}}async showShareModal(){let e=await this.appServices?.get(),t=await e?.analyticsHandler.get();if(!t)return;this.modal=await e?.modalManager.get();let n=i`
      <share-modal-content
        .url=${this.url}
        .subject=${this.title}
        .mediatype=${this.mediaType}
        .identifier=${this.identifier}
        .description=${`Search for "${this.title}" on the Internet Archive.`}
        .analyticsHandler=${t}
        .analyticsCategory=${this.analyticsCategory}
      >
      </share-modal-content>
    `,r=i`<p>${a(`Share`)}</p>`,o=new u;o.processingImageMode=`complete`,o.title=r,o.headerColor=`#194880`,o.showHeaderLogo=!1,this.modal?.showModal({config:o,customModalContent:n,userClosedModalCallback:()=>{this.modal?.removeAttribute(`id`)}}),this.modal?.classList.add(`share-modal-content`)}async favoritesClickHandler(){if(this.mediaType===`search`&&!this.loggedIn){let e=await(await this.appServices?.get())?.modalManager.get();if(!e||!this.appServices)return;v(e,this.appServices,async()=>{this.loggedIn=!0,await this.performFavoriteToggle()});return}await this.performFavoriteToggle()}async performFavoriteToggle(){let e=await(await this.appServices?.get())?.favoriteService?.get();if(!e||!this.identifier)return;let t=this.hasFavorite;this.clearFavoriteError(),this.toggleFavoriteStatus();try{(t?await e.removeFavorite({identifier:this.identifier,mediatype:this.mediaType}):await e.addFavorite({identifier:this.identifier,mediatype:this.mediaType,title:this.title})).success===!1&&this.revertFavoriteWithError()}catch(e){this.revertFavoriteWithError(),console.log(e)}}redirectToManagePage(){let e=new URL(window.location.href);e.pathname=`/search.php`,e.searchParams.set(`manage`,`1`),window.location.assign(e)}playAllPressed(){this.dispatchEvent(new CustomEvent(`playAllPressed`))}toggleFavoriteStatus(){this.hasFavorite=!this.hasFavorite}revertFavoriteWithError(){this.toggleFavoriteStatus(),this.favoriteError=!0,window.clearTimeout(this.favoriteErrorTimer),this.favoriteErrorTimer=window.setTimeout(()=>this.clearFavoriteError(),x)}clearFavoriteError(){window.clearTimeout(this.favoriteErrorTimer),this.favoriteErrorTimer=void 0,this.favoriteError=!1}removeItemsPressed(){this.dispatchEvent(new CustomEvent(`removeItemsPressed`))}render(){return i`
      <div id="link-design">
        <ul>
          <li>${this.manageSettingsTemplate}</li>
          <li>${this.showShare?this.shareTemplate:n}</li>
          <li>${this.showFavoriteButton?this.favTemplate:n}</li>
          <li>${this.rssButtonTemplate}</li>
          <li>${this.editButtonTemplate}</li>
          <li>${this.editxmlButtonTemplate}</li>
          <li>${this.removeItemsButtonTemplate}</li>
          <li>${this.historyButtonTemplate}</li>
          <li>${this.playAllButtonTemplate}</li>
          <li>${this.manageButtonTemplate}</li>
        </ul>
      </div>
    `}get shareTemplate(){return i`
      <button
        id="share"
        class="action-bar-text"
        @click=${()=>this.showShareModal()}
      >
        <ia-icon-label>
          <ia-icon slot="icon" src=${h}></ia-icon>
          <span class="hidden-xs-span">${a(`Share`)}</span>
        </ia-icon-label>
      </button>
    `}get favTemplate(){return i`
      <button
        id="favorite"
        class="action-bar-text ${this.favoriteError?`error`:``}"
        title="${this.favoriteTitle}"
        @click=${this.favoritesClickHandler}
      >
        <ia-icon-label>
          <ia-icon
            slot="icon"
            src=${this.hasFavorite?d:p}
          ></ia-icon>
          <span class="hidden-xs-span">${this.favoriteLabel}</span>
        </ia-icon-label>
      </button>
      <span class="sr-only" role="status">
        ${this.favoriteError?a(`Update failed`):n}
      </span>
    `}get favoriteTitle(){return this.favoriteError?a(`Update failed`):this.hasFavorite?a(`Unfavorite`):a(`Favorite`)}get favoriteLabel(){return this.favoriteError?a(`Update failed`):this.hasFavorite?a(`Unfavorite`):a(`Favorite`)}get rssButtonTemplate(){return!this.showRssButton||!this.rssUrl?n:i`
      <a href="${this.rssUrl}" class="action-bar-text rss">
        <ia-icon-label>
          <ia-icon slot="icon" src=${g}></ia-icon>
          <span class="hidden-xs-span">${a(`RSS`)}</span>
        </ia-icon-label>
      </a>
    `}get playAllButtonTemplate(){return this.showPlayAllButton?i`
      <button class="action-bar-text play-all" @click=${this.playAllPressed}>
        <ia-icon-label>
          <ia-icon slot="icon" src=${y}></ia-icon>
          <span class="hidden-xs-span">${a(`Play All`)}</span>
        </ia-icon-label>
      </button>
    `:n}get editButtonTemplate(){return!this.showEditButton||this.baseNavigationUrl==null?n:i`
      <a
        href=${`${this.baseNavigationUrl}/edit/${this.identifier}`}
        class="action-bar-text edit"
      >
        <ia-icon-label>
          <ia-icon slot="icon" src=${_}></ia-icon>
          <span class="hidden-xs-span">${a(`Edit`)}</span>
        </ia-icon-label>
      </a>
    `}get editxmlButtonTemplate(){return!this.showEditxmlButton||this.baseNavigationUrl===void 0?n:i`
      <a
        href=${`${this.baseNavigationUrl}/editxml/${this.identifier}`}
        class="action-bar-text editxml"
      >
        <ia-icon-label>
          <ia-icon slot="icon" src=${_}></ia-icon>
          <span class="hidden-xs-span">${a(`Edit`)}</span>
        </ia-icon-label>
      </a>
    `}get removeItemsButtonTemplate(){return this.showRemoveItemsButton?i`
      <button
        class="action-bar-text remove-items"
        @click=${this.removeItemsPressed}
      >
        <ia-icon-label>
          <ia-icon slot="icon" src=${b}></ia-icon>
          <span class="hidden-xs-span">${a(`Remove items`)}</span>
        </ia-icon-label>
      </button>
    `:n}get historyButtonTemplate(){return!this.showHistoryButton||this.baseNavigationUrl==null?n:i`
      <a
        href=${`${this.baseNavigationUrl}/history/${this.identifier}`}
        class="action-bar-text history"
      >
        <ia-icon-label>
          <ia-icon slot="icon" .src=${m}></ia-icon>
          <span class="hidden-xs-span">${a(`History`)}</span>
        </ia-icon-label>
      </a>
    `}get manageButtonTemplate(){return this.showManageButton?i`
      <button class="action-bar-text" @click=${()=>this.removeItemsPressed()}>
        <ia-icon-label>
          <ia-icon slot="icon" src=${f}></ia-icon>
          <span class="hidden-xs-span">${a(`Manage items`)}</span>
        </ia-icon-label>
      </button>
    `:n}get manageSettingsTemplate(){return this.showSettingsButton?i`
      <a
        href=${`${this.baseNavigationUrl}/account/settings`}
        class="action-bar-text history"
      >
        <ia-icon-label>
          <ia-icon slot="icon" src=${f}></ia-icon>
          <span class="hidden-xs-span">${a(`Settings`)}</span>
        </ia-icon-label>
      </a>
    `:n}static get styles(){return e`
      #link-design {
        text-align: left;
      }

      ia-icon-label {
        cursor: pointer;
      }

      ul {
        margin: 0;
        padding: 0;
        line-height: normal;
        list-style-type: none;
        display: flex;
        flex-direction: column;
      }

      .action-bar-text {
        display: inline-block;
        background: none;
        padding: 4px 5px;
        border: none;
        text-decoration: none;
        font: inherit; /* Ensure all font properties are inherited by the action buttons */
        color: inherit;
      }

      .action-bar-text.error {
        /*
         * Sit the theme red on a white pill so it stays readable on every
         * host's background (grey search bar, dark profile header, custom
         * collection headers)
         */
        color: var(--ia-theme-color-danger, #e51c23);
        background-color: var(--ia-theme-secondary-background-color, #fff);
        border-radius: 4px;
        padding: 4px 5px;
      }

      .sr-only {
        position: absolute;
        width: 1px;
        height: 1px;
        margin: -1px;
        padding: 0;
        border: 0;
        overflow: hidden;
        clip: rect(0, 0, 0, 0);
        white-space: nowrap;
      }

      .rss ia-icon,
      .play-all ia-icon {
        /* Small adjustment to make the icon visual sizes more consistent */
        --iconWidth: 15px;
      }

      @media (max-width: 767px) {
        #link-design {
          text-align: right;
        }

        .action-bar-text {
          padding: 4px 0;
        }

        ia-icon-label span,
        .hidden-xs-span {
          position: absolute;
          left: -9999px;
          width: 1px;
          height: 1px;
          margin: 0;
          padding: 0;
          border: none;
          overflow: hidden;
        }
      }
    `}};l([t({type:String})],S.prototype,`url`,void 0),l([t({type:String})],S.prototype,`analyticsCategory`,void 0),l([t({type:String})],S.prototype,`mediaType`,void 0),l([t({type:String})],S.prototype,`title`,void 0),l([t({type:String})],S.prototype,`kind`,void 0),l([t({type:String})],S.prototype,`identifier`,void 0),l([t({type:Object})],S.prototype,`modal`,void 0),l([t({type:Object})],S.prototype,`appServices`,void 0),l([t({type:Boolean})],S.prototype,`loggedIn`,void 0),l([t({type:Boolean})],S.prototype,`showShare`,void 0),l([t({type:Boolean})],S.prototype,`showFavoriteButton`,void 0),l([t({type:Boolean})],S.prototype,`showRssButton`,void 0),l([t({type:Boolean})],S.prototype,`showPlayAllButton`,void 0),l([t({type:Boolean})],S.prototype,`showEditButton`,void 0),l([t({type:Boolean})],S.prototype,`showEditxmlButton`,void 0),l([t({type:Boolean})],S.prototype,`showRemoveItemsButton`,void 0),l([t({type:Boolean})],S.prototype,`showHistoryButton`,void 0),l([t({type:Boolean})],S.prototype,`showManageButton`,void 0),l([t({type:Boolean})],S.prototype,`showSettingsButton`,void 0),l([t({type:String})],S.prototype,`baseNavigationUrl`,void 0),l([t({type:String})],S.prototype,`rssUrl`,void 0),l([t({type:String})],S.prototype,`manageActionText`,void 0),l([o()],S.prototype,`hasFavorite`,void 0),l([o()],S.prototype,`favoriteError`,void 0),S=l([c(`action-bar`),r()],S);