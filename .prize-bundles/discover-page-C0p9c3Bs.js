import{A as e,C as t,E as n,N as r,O as i,P as a,S as o,T as s,_ as c,p as l,w as u,x as d}from"./vendor-lit-MQ4DzmMc.js";import{t as f}from"./index-NFRVgqV2.js";import"./log-Bvw5tZC3.js";import{a as p}from"./dist-Nfn4GWEx.js";import"./action-bar-DEhD4wk6.js";import"./collection-search-input-Bb1cFW0Z.js";import{t as m}from"./tile-carousel-DIjqvyrk.js";function h(){return{texts:a(`Books/Texts`),fts:a(`Text Inside`),rcs:a(`Radio Transcripts`),tvs:a(`TV Captions`),whisper:a(`Media Transcripts`),movies:a(`Video Items`),audio:a(`Audio Items`),software:a(`Software Items`),image:a(`Images`),etree:a(`Live Music Concerts`),collection:a(`Collections`),data:a(`Data Items`),web:a(`Web Sites`)}}var g=[`texts`,`fts`,`audio`,`rcs`,`whisper`,`movies`,`tvs`,`software`,`image`,`etree`],_={fts:`texts`,rcs:`radio`,tvs:`tv`},v={fts:`TXT`,rcs:`RADIO`,tvs:`TV`},y={fts:`snippets-only`,rcs:`snippets-only`,tvs:`snippets-only`,whisper:`snippets-only`},b=class extends s{constructor(...e){super(...e),this.useMobileView=!1,this.initialResultsPageLoading=!1,this.federatedResultsLoading=!1,this.displayType=`carousel`,this.analyticsNamespace=`Discover`}render(){return i`
      <div id="page-container">
        <div id="search-bar-container">
          <div
            id="search-bar-content-container"
            .class="${this.useMobileView?`mobile`:n}"
          >
            ${this.useMobileView?n:i`
                  <div id="search-name-container">
                    <h1>${a(`Search`)}</h1>
                  </div>
                `}
            <collection-search-input
              suppressDefaultSearchAction
              hideDropdownOptions
              .query=${this.searchQuery}
              .navBaseUrl=${this.baseNavigationUrl}
              .waybackSearchHandler=${this.waybackSearchHandler}
              .tvSearchHandler=${this.tvSearchHandler}
              ?useMobileView=${this.useMobileView}
              ?showLoadingIndicator=${this.initialResultsPageLoading}
              .selectedSearchOption=${`metadata`}
              @searchRequested=${this.searchRequested}
            >
              ${this.useMobileView?this.actionBarTemplate:n}
            </collection-search-input>
            ${this.useMobileView?n:this.actionBarTemplate}
          </div>
        </div>

        <div id="results-container">${this.resultsTemplate}</div>
      </div>
    `}get resultSections(){return this.federatedResults?g.map(e=>({key:e,heading:h()[e]??`Other Results`,tiles:this.federatedResults?.[e].map(e=>new p(e))??[]})).filter(e=>e.tiles.length>0):[]}displayTypeChanged(e){let t=e.target;setTimeout(()=>this.displayType=t.value,0)}get displayOptionsTemplate(){return i`
      <fieldset id="display-options">
        <legend>${a(`Display type (for prototyping convenience)`)}</legend>
        <label>
          <input
            type="radio"
            name="display"
            value="carousel"
            checked
            @change=${this.displayTypeChanged}
          />
          ${a(`Carousels`)}
        </label>
        <label>
          <input
            type="radio"
            name="display"
            value="list"
            @change=${this.displayTypeChanged}
          />
          ${a(`Lists`)}
        </label>
        <label>
          <input
            type="radio"
            name="display"
            value="list-compact"
            @change=${this.displayTypeChanged}
          />
          ${a(`Compact Lists`)}
        </label>
        <label>
          <input
            type="radio"
            name="display"
            value="grid"
            @change=${this.displayTypeChanged}
          />
          ${a(`Grids`)}
        </label>
      </fieldset>
    `}get resultsTemplate(){if(this.federatedResultsLoading)return this.placeholderGridTemplate;if(!this.searchQuery||!this.federatedResults)return this.emptyPlaceholderTemplate;let e=e=>{let t=_[e]??e,n=v[e]?`&sin=${v[e]}`:``;return i`
        <a
          class="results-show-all-btn"
          href="/labs/search?query=${this.searchQuery}&tab=${t}${n}"
        >
          ${a(`Show More...`)}
        </a>
      `},{resultSections:t}=this;return i`
      ${this.displayOptionsTemplate}
      ${l(t,({key:e})=>`${e}-${this.displayType}`,({key:t,heading:n,tiles:r})=>i`
          <div class="tile-carousel-container ${this.displayType}">
            <h2 class="results-section-heading">${n}</h2>
            ${e(t)}
            <tile-carousel
              class="${this.displayType} wide"
              ?enableHoverPanes=${this.displayType!==`list`}
              .tiles=${r}
              .tileType=${this.tileType}
              .baseNavigationUrl=${this.baseNavigationUrl}
              .baseImageUrl=${this.baseImageUrl}
              .resizeObserver=${this.resizeObserver}
              .loggedIn=${!!this.loggedInUser?.itemname}
              .analyticsHandler=${this.analyticsHandler}
              .layoutType=${y[t]??`stats-only`}
            ></tile-carousel>
          </div>
        `)}
    `}get tileType(){switch(this.displayType){case`list`:return`list-detail`;case`list-compact`:return`list-compact`;default:return`grid`}}get hasEmptyResults(){return!!this.federatedResults&&Object.values(this.federatedResults).every(e=>e.length===0)}get emptyPlaceholderTemplate(){let e=null;return this.searchQuery?this.hasEmptyResults&&(e=`no-results`):e=`empty-query`,i`
      <empty-placeholder
        .placeholderType=${e}
        .isMobileView=${this.useMobileView}
      ></empty-placeholder>
    `}get placeholderGridTemplate(){return i`
      <tile-carousel
        class="grid wide"
        usePlaceholder
        .numPlaceholders=${50}
      ></tile-carousel>
    `}get actionBarTemplate(){return i`
      <div
        id="action-bar-spacing"
        slot=${this.useMobileView?`search-links`:``}
      >
        <action-bar
          .appServices=${this.appServices}
          .url=${`${this.currentBaseUrl}/discover?query=${this.searchQuery}`}
          .title=${this.searchQuery??``}
          .analyticsCategory=${this.analyticsNamespace}
          .mediaType=${`search`}
          .identifier=${this.searchQuery}
          .titleSearch=${this.currentBaseUrl}
          ?showFavoriteButton=${!!this.loggedInUser}
        ></action-bar>
      </div>
    `}handleFederatedResultsLoaded(){}async populateFederatedResults(){this.searchQuery&&(this.federatedResultsLoading=!0,this.federatedResults=await m(this.searchQuery,this.searchService),this.federatedResultsLoading=!1,this.handleFederatedResultsLoaded())}async searchRequested(e){this.searchQuery=e.detail,this.federatedResults=void 0,this.searchQuery&&this.populateFederatedResults(),this.setupPageMetadata(),this.sendAnalyticsEvent({action:`SearchRequested`})}sendAnalyticsEvent(e){this.analyticsHandler?.sendEvent({category:this.analyticsNamespace,action:e.action,label:e.label})}disconnectedCallback(){this.resizeObserver&&this.disconnectResizeObserver(this.resizeObserver)}handleResize(e){e.target===this.searchBarContentContainer&&(this.useMobileView=e.contentRect.width<600)}firstUpdated(){this.sendAnalyticsEvent({action:`PageView`});let e=new URL(window.location.href).searchParams.get(`query`);e&&(this.searchQuery=e,this.populateFederatedResults())}updated(e){if(e.has(`appServices`)&&this.appServices&&this.setupAppServicesDependents(),e.has(`searchQuery`)&&this.setupPageMetadata(),e.has(`resizeObserver`)){let t=e.get(`resizeObserver`);t&&this.disconnectResizeObserver(t),this.setupResizeObserver()}}disconnectResizeObserver(e){e.removeObserver({target:this.searchBarContentContainer,handler:this})}setupResizeObserver(){this.resizeObserver&&this.resizeObserver.addObserver({target:this.searchBarContentContainer,handler:this})}async setupAppServicesDependents(){let e=await this.appServices?.get();await Promise.all([this.searchService=await e?.searchService.get(),this.resizeObserver=await e?.resizeObserver.get(),this.waybackSearchHandler=await e?.waybackSearchHandler.get(),this.tvSearchHandler=await e?.tvSearchHandler.get(),this.analyticsHandler=await e?.analyticsHandler.get(),this.pageMetadataService=await e?.pageMetadataService.get(),this.baseNavigationUrl=(await e?.appConfig.get())?.archiveNavUrl,this.baseImageUrl=(await e?.appConfig.get())?.baseImageUrl,this.currentBaseUrl=(await e?.appConfig.get())?.currentBaseUrl,this.setupLoggedInUser(),this.setupPageMetadata()])}async setupLoggedInUser(){let e=await(await this.appServices?.get())?.userService.get();this.loggedInUser=(await e?.getLoggedInUser())?.success}async setupPageMetadata(){let e=new URL(window.location.href);this.searchQuery?e.searchParams.set(`query`,this.searchQuery):e.searchParams.delete(`query`),window.history.pushState(null,``,e),await Promise.all([this.pageMetadataService?.clearLinkTags(),this.pageMetadataService?.setMetaTag({name:`robots`,content:`noindex, nofollow`}),this.pageMetadataService?.addLink({rel:`canonical`,path:`/discover${c(this.searchQuery,()=>`?query=${this.searchQuery}`)}`})])}static get styles(){return e`
      search-bar {
        height: 100px;
      }

      #page-container {
        font-size: 1.4rem;
        margin: 0 0 30px;
      }

      a:link,
      a:visited {
        text-decoration: none;
        color: var(--ia-theme-link-color);
      }

      a:hover {
        text-decoration: underline;
      }

      #results-container {
        max-width: var(--app-page-max-width);
        margin: auto;
        padding: 20px 0;
      }

      #display-options {
        margin: 0 10px 10px;
      }

      #display-options > label {
        margin-right: 10px;
      }

      #search-bar-container {
        position: relative;
        background-color: #d8d8d8;
        padding: 40px 15px 15px 15px;
        z-index: 3;
      }

      #search-name-container {
        margin-right: 1rem;
      }

      #search-name-container h1 {
        margin: 0;
        line-height: 1.1;
      }

      #search-bar-content-container {
        max-width: 1000px;
        margin: auto;
        display: flex;
      }

      #search-bar-content-container.mobile {
        display: block;
      }

      collection-search-input {
        flex: 1;
      }

      #action-bar-spacing {
        margin-left: 1rem;
        font-size: 1.3rem;
        height: fit-content;
      }

      .results-section-heading {
        display: inline-block;
        margin: 0 0 5px 0;
        padding-left: 9px;
        font-size: 2.2rem;
        line-height: 25px;
      }
      .results-section-heading:not(:first-child) {
        margin-top: 10px;
      }

      .results-show-all-btn {
        appearance: none;
        color: var(--ia-theme-link-color);
        background: transparent;
        border: 0;
        cursor: pointer;
        font-size: 15px;
        padding: 5px 10px 0;
        text-decoration: none;
      }

      .tile-carousel-container {
        margin: 0 10px 15px;
        padding: 5px 0;
        background-color: #eaf2fb;
        border: 1px solid var(--secondaryCTABorder, #999);
        border-radius: 10px;
      }

      .tile-carousel-container.list,
      .tile-carousel-container.list-compact {
        background-color: transparent;
      }

      .tile-carousel-top-row {
        display: flex;
        align-items: center;
        column-gap: 5px;
      }

      #display-options {
        display: inline-block;
      }

      tile-carousel.grid,
      tile-carousel.carousel {
        --imgBlockHeight: 8rem;
      }

      tile-carousel.list {
        display: flex;
        flex-direction: column;
        row-gap: 25px;
        overflow: hidden;
      }

      tile-carousel.list-compact {
        display: flex;
        flex-direction: column;
        row-gap: 10px;
        overflow: hidden;
      }

      .carousel-show-more-btn {
        appearance: none;
        color: var(--ia-theme-link-color);
        background: transparent;
        border: 0;
        cursor: pointer;
        font-size: 15px;
        margin-left: 5px;
        padding: 0;
        text-decoration: none;
      }
    `}};f([t({type:String})],b.prototype,`searchQuery`,void 0),f([t({type:Object})],b.prototype,`appServices`,void 0),f([o()],b.prototype,`waybackSearchHandler`,void 0),f([o()],b.prototype,`tvSearchHandler`,void 0),f([o()],b.prototype,`searchService`,void 0),f([o()],b.prototype,`resizeObserver`,void 0),f([o()],b.prototype,`baseNavigationUrl`,void 0),f([o()],b.prototype,`baseImageUrl`,void 0),f([o()],b.prototype,`currentBaseUrl`,void 0),f([o()],b.prototype,`useMobileView`,void 0),f([o()],b.prototype,`loggedInUser`,void 0),f([o()],b.prototype,`initialResultsPageLoading`,void 0),f([o()],b.prototype,`federatedResults`,void 0),f([o()],b.prototype,`federatedResultsLoading`,void 0),f([o()],b.prototype,`displayType`,void 0),f([d(`#search-bar-content-container`)],b.prototype,`searchBarContentContainer`,void 0),b=f([u(`discover-page`),r()],b);var x=b;export{x as default};