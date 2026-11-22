import{A as e,C as t,O as n,S as r,T as i,w as a,x as o}from"./vendor-lit-MQ4DzmMc.js";import{g as s,t as c}from"./index-NFRVgqV2.js";import{t as l}from"./theme-option-BP5-K2Y3.js";import{n as u,r as d,t as f}from"./no-news-posts-override-B2Yx8f-o.js";var p=class extends i{constructor(...e){super(...e),this.expanded=!1,this.activityIndicatorRequestCount=0}render(){return n`
      <div
        id="page-positioning-container"
        class="${this.expanded?`expanded`:``}"
      >
        <div id="tray-container" tabindex="-1">
          <div id="tray">
            <dl>
              <dt>Language</dt>
              <dd>
                <button @click=${this.useEnglish}>EN</button>
                <button @click=${this.useSpanish}>ES</button>
              </dd>
              <dt>Theme</dt>
              <dd>
                <button @click=${this.useThemeLight}>Light</button>
                <button @click=${this.useThemeDark}>Dark</button>
                <button @click=${this.useThemeAuto}>Auto</button>
              </dd>
              <dt>Page Activity Indicator</dt>
              <dd>
                <button @click=${this.showPageActivityIndicator}>Show</button>
                <button @click=${this.hidePageActivityIndicator}>Hide</button>
                <span>Requests: ${this.activityIndicatorRequestCount}</span>
              </dd>
              <dt>Page Max Width</dt>
              <dd>
                <input
                  type="range"
                  min="32"
                  max="150"
                  value="117"
                  step="0.1"
                  aria-label="Adjust width"
                  @input=${this.widthSliderChanged}
                />
                <span id="pageWidthSize">117rem</span>
              </dd>
              <dt>Top Nav Search</dt>
              <dd>
                <button @click=${this.showTopNavSearch}>Show</button>
                <button @click=${this.hideTopNavSearch}>Hide</button>
              </dd>
              <dt>Top Nav 2nd Logo</dt>
              <dd>
                <button @click=${this.showTopNav2ndLogo}>Show</button>
                <button @click=${this.hideTopNav2ndLogo}>Hide</button>
              </dd>
              <dt>Zendesk Widget</dt>
              <dd>
                <button @click=${this.showZendeskWidget}>Show</button>
                <button @click=${this.hideZendeskWidget}>Hide</button>
              </dd>
              <dt>Throw Test Error</dt>
              <dd>
                <button @click=${this.throwError}>Throw</button>
              </dd>
              <dt>Add allowed custom elements</dt>
              <dd>
                <button @click=${this.addAllowedCustomElementsToPage}>
                  Add
                </button>
              </dd>
              <dt>Reset Lists callouts (needs reload)</dt>
              <dd>
                <button @click=${this.resetListsCalloutsDismissed}>
                  Reset
                </button>
              </dd>
              <dt>Donation: Add Venmo Option</dt>
              <dd>
                <button @click=${this.addDonationVenmoOption}>Add</button>
              </dd>
              <dt>No News Posts (homepage Archive News)</dt>
              <dd>
                <button @click=${this.enableNoNewsPostsOverride}>On</button>
                <button @click=${this.disableNoNewsPostsOverride}>Off</button>
              </dd>
            </dl>
          </div>
          <div id="tab" @click=${this.tabClicked}>
            <div id="tab-text">DEV</div>
          </div>
        </div>
      </div>
    `}tabClicked(){this.expanded=!this.expanded}async widthSliderChanged(e){let t=e.target.value;this.pageWidthSize.innerText=`${t}rem`,document.documentElement.style.setProperty(`--app-page-max-width`,t+`rem`)}async throwError(){throw Error(`This is a test error`)}async useEnglish(){await(await(await this.appServices?.get())?.localizationManager.get())?.setLanguage(s.en,!0)}async useSpanish(){await(await(await this.appServices?.get())?.localizationManager.get())?.setLanguage(s.es,!0)}async useThemeLight(){(await(await this.appServices?.get())?.themeManager?.get())?.setTheme(l.light,!0)}async useThemeDark(){(await(await this.appServices?.get())?.themeManager?.get())?.setTheme(l.dark,!0)}async useThemeAuto(){(await(await this.appServices?.get())?.themeManager?.get())?.setTheme(l.auto,!0)}async showTopNavSearch(){this.toggleTopNavSearch(!0)}async hideTopNavSearch(){this.toggleTopNavSearch(!1)}async toggleTopNavSearch(e){(await(await this.appServices?.get())?.appRootController.get())?.showTopNavSearch(e)}async showTopNav2ndLogo(){this.toggleTopNav2ndLogo(!0)}async hideTopNav2ndLogo(){this.toggleTopNav2ndLogo(!1)}async toggleTopNav2ndLogo(e){(await(await this.appServices?.get())?.appRootController.get())?.showTopNav2ndLogo(e)}async showZendeskWidget(){this.setZendeskVisibility(!0)}async hideZendeskWidget(){this.setZendeskVisibility(!1)}async setZendeskVisibility(e){(await(await this.appServices?.get())?.appRootController.get())?.showZendeskWidget(e)}async showPageActivityIndicator(){this.togglePageActivityIndicator(!0)}async hidePageActivityIndicator(){this.togglePageActivityIndicator(!1)}async addAllowedCustomElementsToPage(){(await(await this.appServices?.get())?.appRootController.get())?.addAllowedCustomElementsToPage()}async resetListsCalloutsDismissed(){let e=await this.appServices?.get(),[t,n]=await Promise.all([e?.keyValueStoreManager.get(),e?.appRootController.get()]),r=t?.getKeyValueStore({namespace:`UserLists`,storageType:`localStorage`});await r?.delete(`tabCalloutDismissed`),await r?.delete(`itemBtnCalloutDismissed`),n?.updateTopNavCallouts()}async addDonationVenmoOption(){let e=(document.querySelector(`app-root`)?.querySelector(`donation-form-controller`)?.querySelector(`donation-form`))?.shadowRoot?.querySelector(`payment-selector`);e.venmoMode=`available`,await e.updateComplete}async enableNoNewsPostsOverride(){await this.setNoNewsPostsOverride(!0)}async disableNoNewsPostsOverride(){await this.setNoNewsPostsOverride(!1)}async setNoNewsPostsOverride(e){await((await(await this.appServices?.get())?.keyValueStoreManager.get())?.getKeyValueStore({namespace:d,storageType:`localStorage`}))?.set(u,e),window.dispatchEvent(new CustomEvent(f))}async togglePageActivityIndicator(e){let t=await(await this.appServices?.get())?.appRootController.get();this.activityIndicatorRequestCount+=e?1:-1,this.activityIndicatorRequestCount=Math.max(0,this.activityIndicatorRequestCount),t?.showActivityIndicator(e)}static get styles(){return e`
      :host {
        --dev-tray-width--: var(--dev-tray-width, 20rem);
        --dev-tray-border-padding--: var(--dev-tray-border-padding, 1rem);
        --dev-tray-border-width--: var(--dev-tray-border-width, 1px);
        --dev-track-background-color--: var(
          --dev-track-background-color,
          var(--ia-theme-primary-background-color, lightgray)
        );
      }

      #page-positioning-container {
        position: fixed;
        z-index: 100;
        top: 10rem;
        left: calc(
          (
              var(--dev-tray-width--) + (var(--dev-tray-border-padding--) * 2) +
                (var(--dev-tray-border-width--) * 2)
            ) *
            -1
        );
        transition: left 0.3s;
      }

      #page-positioning-container.expanded {
        left: 0;
      }

      #tray-container {
        display: flex;
        align-items: flex-start;
        height: 40px;
      }

      #tray {
        background-color: var(--dev-track-background-color--);
        border-style: solid;
        border-color: darkgray;
        border-width: var(--dev-tray-border-width--);
        border-left: 0;
        width: var(--dev-tray-width--);
        border-radius: 0 0 5px 0;
        padding: 0.5rem 1rem;
        box-shadow: 0px 0px 5px rgba(0, 0, 0, 0.3);
        z-index: 1;
      }

      dt {
        font-size: 1.2rem;
        margin: 0 0 0.3rem 0;
      }

      dd {
        margin-left: 0;
        margin-bottom: 0.7rem;
      }

      #tab {
        font-size: 1.3rem;
        border-top: 1px solid darkgray;
        border-radius: 0 5px 5px 0;
        border-right: 1px solid darkgray;
        border-bottom: 1px solid darkgray;
        background-color: var(--dev-track-background-color--);
        height: 3em;
        margin-left: -1px;
        cursor: pointer;
        box-shadow: 0px 0px 5px rgba(0, 0, 0, 0.3);
      }

      #tab-text {
        transform: rotate(90deg) translate(50%, 0);
      }
    `}};c([t({type:Object})],p.prototype,`appServices`,void 0),c([r()],p.prototype,`expanded`,void 0),c([o(`#pageWidthSize`)],p.prototype,`pageWidthSize`,void 0),c([r()],p.prototype,`activityIndicatorRequestCount`,void 0),p=c([a(`dev-tools`)],p);export{p as DevTools};