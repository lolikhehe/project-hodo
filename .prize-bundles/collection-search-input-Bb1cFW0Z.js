import{A as e,C as t,E as n,N as r,O as i,P as a,T as o,m as s,s as c,w as l,x as u}from"./vendor-lit-MQ4DzmMc.js";import{t as d}from"./index-NFRVgqV2.js";import"./ia-status-indicator-Bd61y7bd.js";import"./ia-clearable-text-input-CaSQalrP.js";import{a as f}from"./dist-P4PnKJsG.js";var p={"":`metadata`,TXT:`fulltext`,RADIO:`radio`,TV:`tv`,WEB:`wayback`,AIT:`archiveit`},m=Object.entries(p).reduce((e,[t,n])=>(e[n]=t,e),{});function h(){return{metadata:a(`Search metadata`),fulltext:a(`Search text contents`),radio:a(`Search radio transcripts`),tv:a(`Search TV news captions`),wayback:a(`Search archived web sites`),archiveit:a(`Search text contents (Archive-It)`)}}var g=class extends o{constructor(...e){super(...e),this.suppressDefaultSearchAction=!1,this.suppressAdvancedSearch=!1,this.selectedSearchOption=`metadata`,this.useMobileView=!1,this.showLoadingIndicator=!1,this.hideDropdownOptions=!1,this.inlineSearchLinks=!1,this.displayedSearchOptions=[{option:`metadata`,enabled:!0},{option:`fulltext`,enabled:!0},{option:`tv`,enabled:!0},{option:`radio`,enabled:!0},{option:`wayback`,enabled:!0}],this.searchOptionsVisible=!1}get resolvedButtonText(){return this.buttonText??a(`GO`)}get resolvedPlaceholderText(){return this.placeholderText??a(`Search`)}render(){let e=this.query?.replace(/\+/g,` `)??``;return e&&(this.searchOptionsVisible=!0),i`
      <div id="container">
        <form
          id="search-form"
          class=${this.inlineSearchLinks?`inline-links`:n}
          method="get"
          role="search"
          .action=${this.searchEndpoint}
          data-event-form-tracking="Home|SearchForm"
          @submit=${this.formSubmitted}
        >
          <div id="input-button">
            <ia-clearable-text-input
              id="search-input"
              .value=${e}
              placeholder=${this.resolvedPlaceholderText}
              aria-owns="search-options"
              clearButtonScreenReaderLabel=${a(`Clear search query`)}
              screenReaderLabel=${a(`Search the Archive. Filters and Advanced Search available below.`)}
              @focus=${this.searchFieldFocused}
              @input=${this.searchFieldInput}
              @clear=${this.searchFieldCleared}
              @submit=${this.submitForm}
            ></ia-clearable-text-input>
            <input
              id="hidden-form-query"
              type="hidden"
              name="query"
              .value=${e}
            />
            <button
              id="go-button"
              class=${this.showLoadingIndicator?`loading`:n}
              type="submit"
              data-button-text=${this.resolvedButtonText}
            >
              ${this.showLoadingIndicator?i` <ia-status-indicator
                    mode="loading"
                    class="go-button-loading-icon"
                  ></ia-status-indicator>`:this.resolvedButtonText}
            </button>
          </div>
          ${this.searchOptionsAndLinksTemplate}
        </form>
      </div>
    `}get advancedSearchTemplate(){if(this.suppressAdvancedSearch)return n;let e=this.advancedSearchUrl;if(!e){if(this.navBaseUrl===void 0)return n;let t=this.query?`?q=${encodeURIComponent(this.query)}`:``;e=`${this.navBaseUrl}/advancedsearch.php${t}`}return i`<a
      id="advanced-search-link"
      href=${e}
      @click=${this.advancedSearchClicked}
    >
      ${a(`Advanced Search`)}
    </a>`}get searchOptionsAndLinksTemplate(){return i`
      <div
        id="search-options"
        class=${s({"is-open":!this.hideDropdownOptions&&this.searchOptionsVisible,"options-hidden":this.hideDropdownOptions,mobile:this.useMobileView,desktop:!this.useMobileView})}
        aria-label="Search Options"
        data-keep-open-when-changed="true"
        @click=${this.userChoseRadioButton}
      >
        <div id="button-container">
          <fieldset id="button-collapser">
            <ul>
              ${c(this.displayedSearchOptions,({option:e,enabled:t})=>i`<li>
                    <label>
                      <input
                        type="radio"
                        name="sin"
                        value=${m[e]}
                        .checked=${this.selectedSearchOption===e}
                        ?disabled=${!t}
                        @click=${this.userChoseRadioButton}
                      />
                      <span>
                        ${h()[e]}
                        ${t?``:a(`(no results)`)}
                      </span>
                    </label>
                  </li>`)}
            </ul>
          </fieldset>

          ${this.useMobileView?this.advancedSearchTemplate:n}
        </div>
        <slot name="before-search-links"></slot>
        <div id="search-links">
          <slot name="search-links-top">
            <div><!--Empty div avoids layout shift from search beta A/B--></div>
          </slot>
          ${this.useMobileView?n:this.advancedSearchTemplate}
          <slot name="search-links"></slot>
        </div>
      </div>
    `}get searchEndpoint(){return`${this.navBaseUrl??``}/search`}searchFieldFocused(){this.searchOptionsVisible=!0}searchFieldInput(){this.hiddenFormQuery.value=this.searchInput.value}searchFieldCleared(){if(this.query){let e=new CustomEvent(`searchRequested`,{detail:this.searchInput.value});this.dispatchEvent(e)}}submitForm(){let e=this.searchForm;e?.requestSubmit?e.requestSubmit():this.goButton?.click()}userChoseRadioButton(e){if(!(e.target instanceof HTMLInputElement))return;let t=e.target.value,n=this.selectedSearchOption;if(this.selectedSearchOption=p[t]??`metadata`,this.selectedSearchOption!==n){let e=new CustomEvent(`searchTypeChanged`,{detail:this.selectedSearchType});this.dispatchEvent(e)}}advancedSearchClicked(){this.dispatchEvent(new CustomEvent(`advancedSearchClicked`))}async formSubmitted(e){let t=this.selectedSearchOption,n={wayback:this.waybackSearchHandler,archiveit:this.archiveItSearchHandler},r=Object.keys(n).includes(t);if((this.suppressDefaultSearchAction||r)&&e.preventDefault(),r){n[t]?.performSearch(this.searchInput.value);return}let i=new CustomEvent(`searchRequested`,{detail:this.searchInput.value});this.dispatchEvent(i)}get selectedSearchType(){switch(this.selectedSearchOption){case`fulltext`:return f.FULLTEXT;case`radio`:return f.RADIO;case`tv`:return f.TV;default:return f.METADATA}}static get styles(){return e`
      :host {
        --input-height: 3rem;
        --search-button-margin: 1.5rem;
        --clear-button-top: 0;
        --clear-button-right: 2px;
      }

      #search-form.inline-links {
        display: flex;
      }

      #input-button {
        display: flex;
        height: var(--input-height);
        margin-bottom: 1rem;
        flex: 1;
      }

      #search-input {
        width: 100%;
        height: 100%;
        margin-inline-end: var(--search-button-margin);

        --input-background-color: #fff;
      }

      #go-button {
        appearance: none;
        border-radius: 20px;
        background-color: #2c2c2c;
        font-family: inherit;
        font-weight: bold;
        font-size: 1.4rem;
        border: 0;
        color: #fff;
        padding: 0 1rem;
        height: 100%;
        white-space: nowrap;
        cursor: pointer;
      }

      #go-button.loading {
        background-color: #2c2c2c33; /* 20% opacity */
      }

      /*
       * Use an invisible pseudo-element to ensure the go button's size
       * won't change when switching between its text and loading icon.
       */
      #go-button::before {
        content: attr(data-button-text);
        display: block;
        height: 0;
        min-width: 35px;
        visibility: hidden;
        white-space: nowrap;
        speak: none;
      }

      .go-button-loading-icon {
        margin-top: 2px;
      }

      #search-options {
        display: flex;
        align-items: flex-start;
        justify-content: space-between;
        margin-top: 0.5em;
        padding-left: 5px;
        font-size: 1.4rem;
      }

      #search-options.options-hidden {
        justify-content: flex-start;
      }

      #search-options ul {
        margin: 0;
        padding: 0;
        list-style: none;
      }

      #search-options li:not(:first-child) {
        margin-top: 0.5rem;
      }

      #search-options input[name='sin']:disabled ~ span {
        color: #767676;
      }

      #button-container {
        flex-grow: 1;
      }

      #button-collapser {
        border: 0;
        padding: 0;
        flex: 1 1 auto;
        overflow-y: hidden;
        margin: 0;
        max-height: 0;
        visibility: hidden;
        transition: max-height 0.3s ease-out;
      }

      .options-hidden #button-collapser {
        display: none;
      }

      #search-links {
        text-align: right;
      }

      #search-links > a:not(:last-child) {
        margin-bottom: 0.2rem;
      }

      #search-options.mobile #search-links {
        flex-basis: var(--searchLinksSpacing, 0);
      }

      #search-options.desktop #search-links {
        display: flex;
        justify-content: space-between;
        column-gap: 10px;
        width: 100%;
        margin-right: var(--searchLinksSpacing, 0);
      }

      #search-options.mobile.options-hidden #search-links,
      #search-options.desktop:not(.options-hidden) #search-links {
        width: auto;
      }

      .inline-links #search-links {
        margin-left: 10px;
      }

      #old-search-link {
        display: block;
        margin-bottom: 0.5rem;
        font-size: 1.4rem;
      }

      .is-open #button-collapser {
        max-height: 25rem;
        visibility: visible;
        margin-bottom: 0.5rem;
      }

      a:link {
        text-decoration: none;
        display: block;
        flex: 0 1 auto;
        color: var(--ia-theme-link-color, #4b64ff);
        white-space: nowrap;
      }

      a:hover {
        text-decoration: underline;
      }

      a:visited {
        color: var(--ia-theme-link-color, #4b64ff);
      }

      ::slotted(#action-bar-spacing) {
        margin-top: 6px;
        height: fit-content;
      }
    `}};d([t({type:String})],g.prototype,`query`,void 0),d([t({type:String})],g.prototype,`navBaseUrl`,void 0),d([t({type:String})],g.prototype,`advancedSearchUrl`,void 0),d([t({type:Object})],g.prototype,`waybackSearchHandler`,void 0),d([t({type:Object})],g.prototype,`archiveItSearchHandler`,void 0),d([t({type:Boolean})],g.prototype,`suppressDefaultSearchAction`,void 0),d([t({type:Boolean})],g.prototype,`suppressAdvancedSearch`,void 0),d([t({type:String})],g.prototype,`selectedSearchOption`,void 0),d([t({type:Boolean})],g.prototype,`useMobileView`,void 0),d([t({type:Boolean})],g.prototype,`showLoadingIndicator`,void 0),d([t({type:Boolean})],g.prototype,`hideDropdownOptions`,void 0),d([t({type:Boolean})],g.prototype,`inlineSearchLinks`,void 0),d([t({type:String})],g.prototype,`buttonText`,void 0),d([t({type:String})],g.prototype,`placeholderText`,void 0),d([t({type:Array})],g.prototype,`displayedSearchOptions`,void 0),d([t({type:Boolean})],g.prototype,`searchOptionsVisible`,void 0),d([u(`#search-form`)],g.prototype,`searchForm`,void 0),d([u(`#search-input`)],g.prototype,`searchInput`,void 0),d([u(`#hidden-form-query`)],g.prototype,`hiddenFormQuery`,void 0),d([u(`#go-button`)],g.prototype,`goButton`,void 0),g=d([l(`collection-search-input`),r()],g);