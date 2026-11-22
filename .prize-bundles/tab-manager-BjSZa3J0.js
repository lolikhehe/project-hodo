import{A as e,C as t,E as n,N as r,O as i,P as a,T as o,m as s,s as c,v as l,w as u}from"./vendor-lit-MQ4DzmMc.js";import{t as d}from"./index-NFRVgqV2.js";var f={pushHistory:!0,force:!1,stripSuffix:!1},p=class{persistTabToURL(e,t=f){let n=this.getResultingURLForTab(e,t);this.updateHistory({tab:e},n,t)}updateHistory(e,t,n=f){n.pushHistory?window.history.pushState(e,``,t):window.history.replaceState(e,``,t)}getURL(){return new URL(window.location.href)}getURLPath(){return this.getURL().pathname}getURLParams(){return this.getURL().searchParams}setURL(e){window.history.replaceState({},``,e)}},m=class extends p{getTabFromURL(){return this.getURLParams().get(`tab`)}getResultingURLForTab(e,t=f){let n=this.getURL();return n.searchParams.get(`tab`)===e&&!t.force||n.searchParams.set(`tab`,e),n}removeTabFromURL(e=f){let t=this.getURL();t.searchParams.delete(`tab`),this.updateHistory({tab:null},t,e)}},h=class extends o{constructor(...e){super(...e),this.identifier=``,this.urlAdapter=new m,this.restoreTabFromURL=()=>{let e=this.activeTabId,t=this.urlAdapter.getTabFromURL();if(!t)this.activeTabId=this.defaultTabId??this.tabs?.[0].id;else if(this.tabs?.find(e=>e.id===t))this.activeTabId=t;else return;this.activeTabId!==e&&this.emitTabChangedEvent()}}render(){return this.tabs?.length?(this.activeTabId===void 0&&(this.activeTabId=this.defaultTabId??this.tabs[0].id),i`
      <div class="tab-manager-container">
        <div class="tabs-outer-container">
          <div class="tabs-container">
            <slot name="tabs-row-left"></slot>
            <div
              class="tabs-row"
              data-testid="tab-manager-tabs-row"
              role="tablist"
              aria-label=${a(`Navigation Tabs`)}
            >
              ${c(this.tabs,this.tabTemplate.bind(this))}
            </div>
          </div>
        </div>
        <div
          class="active-tab-content"
          id="active-tab-content"
          role="tabpanel"
          aria-label="${l(this.activeTab?.displayName)} Tab Content"
        >
          <slot name="${this.activeTab?.id??n}"></slot>
        </div>
      </div>
    `):i`${n}`}connectedCallback(){super.connectedCallback(),this.setupStateRestorationObserver()}disconnectedCallback(){super.disconnectedCallback(),this.teardownStateRestorationObserver()}firstUpdated(){this.restoreTabFromURL()}updated(e){(e.has(`tabs`)||e.has(`activeTabId`))&&this.horizontalScrollToActiveTab()}setupStateRestorationObserver(){window.addEventListener(`popstate`,this.restoreTabFromURL)}teardownStateRestorationObserver(){window.removeEventListener(`popstate`,this.restoreTabFromURL)}tabTemplate(e){let{id:t,displayName:r,callout:o,url:c,count:l}=e,u=this.isActiveTab(e),d=s({tab:!0,active:u}),f=l?`${r} (${l})`:r,p=o?`${a(`New feature: `)}${f}`:f,m=l?i`${r} <span class="tab-count">(${l})</span>`:r;return c?i`
        <a
          class="tab"
          href=${c}
          data-tab-id=${t}
          data-testid=${t}
          data-sizing=${f}
          role="tab"
          aria-label=${p}
          @click=${e=>this.handleLinkTabClick(t,e)}
        >
          ${m}
        </a>
      `:i`
      <button
        data-tab-id=${t}
        data-testid=${t}
        data-sizing=${f}
        role="tab"
        aria-label=${p}
        aria-selected=${u}
        aria-controls="active-tab-content"
        class=${d}
        @click=${e=>this.handleTabClick(t,e)}
      >
        ${m}
      </button>

      ${o?this.calloutTemplate(e):n}
    `}calloutTemplate(e){let{id:t,callout:n}=e;return i`
      <button
        class="callout"
        aria-hidden="true"
        @click=${()=>this.handleCalloutClick(t)}
      >
        <span class="callout-text">${n}</span>
      </button>
    `}handleTabClick(e,t){t.preventDefault(),t.stopPropagation(),this.selectTab(e)}selectTab(e){let t=this.tabs?.find(t=>t.id===e);if(!t||t.url)return;let n=this.activeTabId;this.activeTabId=e,this.activeTabId!==n&&(this.persistActiveTabToURL(),this.emitTabChangedEvent())}handleLinkTabClick(e,t){t.preventDefault(),this.dispatchEvent(new CustomEvent(`linkTabClicked`,{detail:e})),setTimeout(()=>{let t=this.tabs?.find(t=>t.id===e);t?.url&&(window.location.href=t.url)})}handleCalloutClick(e){this.dispatchEvent(new CustomEvent(`calloutClicked`,{detail:e}))}emitTabChangedEvent(){this.dispatchEvent(new CustomEvent(`tabChanged`,{detail:this.activeTabId}))}get activeTab(){return this.tabs?.find(e=>this.isActiveTab(e))}isActiveTab(e){return this.activeTabId===e.id}persistActiveTabToURL(){let e=this.activeTabId;e&&this.urlAdapter.persistTabToURL(e)}horizontalScrollToActiveTab(){let e=this.shadowRoot?.querySelector(`li.tab.active`),t=e?.offsetLeft,n=e?.offsetWidth,r=this.shadowRoot?.querySelector(`.tabs-row`);r.scrollLeft=t-r?.offsetWidth/2+n/2}static get styles(){return e`
      :host {
        --tab-manager-underline-height--: var(
          --tab-manager-underline-height,
          2px
        );
      }

      .tabs-outer-container {
        margin-bottom: 10px;
        border-bottom: var(--tab-manager-tabs-row-border-bottom, 0);
        background: var(
          --tab-manager-tabs-row-background,
          var(--ia-theme-secondary-background-color, white)
        );
      }

      .tabs-container {
        margin: 0 auto;
        padding: 0 5px;
        max-width: var(--app-page-max-width, 1350px);
      }

      :host(.tabs-hidden) .tabs-row {
        visibility: hidden;
      }

      .tabs-row {
        background: var(
          --tab-manager-tabs-row-background,
          var(--ia-theme-secondary-background-color, white)
        );
        padding: var(--tab-manager-tabs-row-padding-top, 25px) 0 0;
        white-space: nowrap;
        overflow-x: scroll;
        overflow-y: hidden;
        -webkit-overflow-scrolling: touch;
        -ms-overflow-style: none; /* IE and Edge */
        scrollbar-width: none; /* Firefox */
        height: var(--tab-manager-tabs-row-height, 28.5px);
        display: flex;
        margin: 0 auto 0;
        column-gap: 10px;
      }

      .tabs-row::-webkit-scrollbar {
        display: none;
      }

      .tab {
        text-align: center;
        margin: 0;
        padding: var(--tab-manager-tab-padding, 5px);
        background: transparent;
        color: var(
          --tab-manager-tab-text-color,
          var(--ia-theme-primary-text-color, #2c2c2c)
        );
        font-size: var(--tab-manager-tab-font-size, 1.4rem);
        font-family: inherit;
        text-decoration: none;
        cursor: pointer;
        border: 0;
        padding-bottom: var(--tab-manager-underline-height--);
      }

      .tab.active {
        border-bottom: var(--tab-manager-underline-height--) solid
          var(
            --tab-manager-tab-text-color,
            var(--ia-theme-primary-text-color, #2c2c2c)
          );

        color: var(
          --tab-manager-tab-text-color,
          var(--ia-theme-primary-text-color, #2c2c2c)
        );
        padding-bottom: 0;
      }

      .tab.active,
      .tab:hover {
        font-weight: bold;
      }

      .tab::before {
        content: attr(data-sizing);
        display: block;
        font-weight: bold;
        height: 0;
        overflow: hidden;
        visibility: hidden;
      }

      a.tab {
        align-self: center;
      }

      .tab-count {
        color: var(--ia-theme-secondary-text-color, #666);
        font-weight: normal;
      }

      .active-tab-content {
        margin: 0 auto;
        padding: 0 5px;
        max-width: var(--app-page-max-width, 1350px);
      }

      .callout {
        position: absolute;
        display: block;
        appearance: none;
        width: 100%;
        top: -25px;
        text-align: center;
        background: #fee257;
        margin: 0;
        padding: 2px 5px;
        border: 0;
        border-radius: 2px;
        font-size: 1.4rem;
        font-weight: bold;
        font-family: inherit;
        box-shadow: 2px 2px 4px rgba(0, 0, 0, 0.8);
        cursor: pointer;
      }

      .callout-text {
        position: relative;
        z-index: 1;
      }

      .callout::after {
        content: '';
        position: absolute;
        width: 8px;
        height: 8px;
        top: 100%;
        left: 50%;
        background: #fee257;
        z-index: 0;
        transform: translateX(-50%) translateY(-50%) rotate(45deg)
          skew(10deg, 10deg);
        box-shadow: 2px 2px 2px rgba(0, 0, 0, 0.8);
      }
    `}};d([t({type:Array})],h.prototype,`tabs`,void 0),d([t({type:String})],h.prototype,`activeTabId`,void 0),d([t({type:String})],h.prototype,`defaultTabId`,void 0),d([t({type:String})],h.prototype,`identifier`,void 0),d([t({type:Object,attribute:!1})],h.prototype,`urlAdapter`,void 0),h=d([u(`tab-manager`),r()],h);export{p as n,f as r,m as t};