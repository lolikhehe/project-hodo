import{A as e,C as t,E as n,F as r,N as i,O as a,P as o,S as s,T as c,_ as l,j as u,l as d,m as f,n as p,s as m,v as h,w as g,x as _}from"./vendor-lit-MQ4DzmMc.js";import{t as v}from"./index-NFRVgqV2.js";import"./ia-dropdown-search-bar-D0po3r-G.js";import"./dist-CTMeFT4b.js";import{a as y}from"./dist-P4PnKJsG.js";import{t as b}from"./format-number-rdMiVR61.js";import{t as x}from"./log-Bvw5tZC3.js";import{n as S}from"./wikipedia-api-service-CXA3ujcu.js";import"./ia-icon-ClrHaxRi.js";import{t as C}from"./capitalize-BPJI7v6S.js";import{t as w}from"./close-DnNtqaAm.js";import{a as T,i as E,n as D,o as O,r as k,t as A}from"./dist-Nfn4GWEx.js";import{t as j}from"./normalize-to-array-B3lD8Ayw.js";import"./action-bar-DEhD4wk6.js";import"./tab-manager-BjSZa3J0.js";import{t as M}from"./back-arrow-BAT84H6i.js";import{t as N}from"./external-link-snEq6XNi.js";import{t as P}from"./tile-carousel-DIjqvyrk.js";import{c as F,i as I,n as L,o as R,r as z,t as B}from"./models-DmFnvYaY.js";import"./toggle-switch-09k58yzO.js";var V=new Set([`de`,`da`,`dos`,`das`,`von`,`van`]),H=/^(?!(?:\bXI)?$)X{0,3}(?:IX|IV|V?I{0,3})$/g,U=/\s|[.'-]|(?=\b)Mc/i,W=U.toString().replace(/^\/|\/[a-z]*$/g,``),G=RegExp(`(?=${W})`,`i`);function K(e){return e.normalize().trim().split(G).map(e=>{if(!e)return``;let t=q(e.match(U)?.[0]??``),n=e.replace(U,``),r=n.toLocaleLowerCase(),i=n.toLocaleUpperCase(),a;return a=V.has(r)?r:H.test(i)?i:q(n),t+a}).join(``)}function q(e){return e.charAt(0).toLocaleUpperCase()+e.slice(1).toLocaleLowerCase()}function J(e){let t=document.createElement(`form`);t.method=e.method??`POST`,t.action=e.action,t.style.display=`none`,t.target=`_top`,document.body.appendChild(t);let n=e.fields??{};Object.keys(n).forEach(e=>{let r=document.createElement(`input`);r.type=`text`,r.name=e;let i=n[e];i&&(r.value=i),t.appendChild(r)}),t.submit()}var Y=class extends c{constructor(...e){super(...e),this.tilesPerService=3,this.maxTiles=12,this.serviceKeys=[`fts`,`tvs`,`rcs`,`whisper`],this.baseImageUrl=`https://archive.org`,this.baseNavigationUrl=``,this.tiles=[],this.size=`normal`,this.layoutType=`default`,this.tilesLoading=!1,this.searchError=!1}render(){return this.searchError?this.searchErrorTemplate:this.carouselTemplate}get carouselTemplate(){return!this.tilesLoading&&this.tiles.length===0?n:a`
      <tile-carousel
        enableHoverPanes
        class=${this.size}
        .layoutType=${this.layoutType}
        .tiles=${this.tiles}
        .usePlaceholder=${this.tilesLoading}
        .baseImageUrl=${this.baseImageUrl}
        .baseNavigationUrl=${this.baseNavigationUrl}
      ></tile-carousel>
    `}get searchErrorTemplate(){return a`<p class="search-error">
      ${o(`No smart results available for this search.`)}
    </p>`}willUpdate(e){e.has(`mediatypeFilter`)||e.has(`layoutType`)||e.has(`serviceKeys`)?this.federatedResults&&this.refreshTiles():e.has(`federatedResults`)&&this.refreshTiles()}get allowedMediatypes(){return this.mediatypeFilter?this.mediatypeFilter:[`texts`,`audio`,`movies`,`etree`,`software`,`image`]}clearTiles(){this.tiles=[],this.searchError=!1}refreshTiles(){this.searchQuery&&(this.clearTiles(),this.buildTilesFromFederatedResults())}buildTilesFromFederatedResults(){let{federatedResults:e}=this;e&&(this.tiles=this.interleavedTiles(e),this.emitResultsLoadedEvent(e))}interleavedTiles(e){let t=[],n=new Set;for(let r=0;r<this.tilesPerService;r++)this.serviceKeys.forEach(i=>{let a=e[i]?.[r],o=a?.identifier,s=a?.mediatype?.value;!o||!s||this.allowedMediatypes.includes(s)&&(n.has(o)||(t.push(new T(a)),n.add(o)))});return t.slice(0,this.maxTiles)}emitResultsLoadedEvent(e){this.dispatchEvent(new CustomEvent(`resultsLoaded`,{detail:e}))}static get styles(){return e`
      .search-error {
        font-size: 1.4rem;
        padding-left: 9px;
      }
    `}};v([t({type:String})],Y.prototype,`searchQuery`,void 0),v([t({type:Object})],Y.prototype,`searchService`,void 0),v([t({type:Number})],Y.prototype,`tilesPerService`,void 0),v([t({type:Number})],Y.prototype,`maxTiles`,void 0),v([t({type:Array})],Y.prototype,`serviceKeys`,void 0),v([t({type:String})],Y.prototype,`baseImageUrl`,void 0),v([t({type:String})],Y.prototype,`baseNavigationUrl`,void 0),v([t({type:Array})],Y.prototype,`tiles`,void 0),v([t({type:Array})],Y.prototype,`mediatypeFilter`,void 0),v([t({type:String})],Y.prototype,`size`,void 0),v([t({type:String})],Y.prototype,`layoutType`,void 0),v([t({type:Object})],Y.prototype,`federatedResults`,void 0),v([t({type:Boolean})],Y.prototype,`tilesLoading`,void 0),v([s()],Y.prototype,`searchError`,void 0),Y=v([g(`federated-search-carousel`),i()],Y);var X={movies:`video`,texts:`text`},Z=class extends c{constructor(...e){super(...e),this.baseImageUrl=`https://archive.org`,this.carouselMinTiles=3,this.carouselMaxTiles=10,this.loading=!1,this.heuristicResults=[]}willUpdate(e){e.has(`searchQuery`)&&this.fetchHeuristics()}async fetchHeuristics(){let e=this.searchQuery??``;if(e===this.heuristicQuery)return;this.heuristicQuery=e;let t=await new A().getRecommendedFacets(e,[D]);e===this.heuristicQuery&&(this.heuristicResults=t)}render(){return this.smartMultiCarouselTemplate()}get allTileModels(){return this.dataSource?Object.values(this.dataSource.getAllPages()).flat():[]}get tilesByMediatype(){let e=this.groupTilesByProperty(`mediatype`);return delete e.texts,delete e.web,e}get tilesByCreator(){return this.groupTilesByProperty(`creators`)}get tilesBySubject(){return this.groupTilesByProperty(`subjects`)}groupTilesByProperty(e){let{allTileModels:t}=this;return t.reduce((t,n)=>{let r=j(n[e]);for(let e of r)!e||typeof e!=`string`||(t[e]||(t[e]=[]),t[e].push(n));return t},{})}filterTilesByProperties(e){let{allTileModels:t}=this;return t.filter(t=>{for(let[n,r]of Object.entries(e))if([`creator`,`subject`].includes(n)){if(!j(t[`${n}s`]).includes(r))return!1}else if(t[n]!==r)return!1;return!0})}heuristicCarouselsTemplate(e){let t=[];for(let r of e){if(r.facets.length===1&&[`mediatype`,`subject`].includes(r.facets[0].facetType))continue;let e=r.facets.reduce((e,{facetType:t,bucketKey:n})=>Object.assign(e,{[t]:n}),{}),i=this.filterTilesByProperties(e);if(i.length===0)continue;let a=r.facets[0],o=r.label??`${a.facetType}: ${a.bucketKey}`,s={renderShowAllBtn:i.length>this.carouselMaxTiles,renderFeedbackWidget:t.length===0},c=this.smartCarouselSection(e,o,i.slice(0,this.carouselMaxTiles),s);c!==n&&t.push(c)}return t}smartMultiCarouselTemplate(){if(!this.dataSource?.size)return n;let{tilesByMediatype:e,tilesByCreator:t,tilesBySubject:r}=this,[i,s,c]=[e,t,r].map(e=>Object.keys(e).sort((t,n)=>e[n].length-e[t].length)),l=s[0],u=c[0],d=t[l],f=r[u],{heuristicResults:p}=this,h=this.heuristicCarouselsTemplate(p),g=p.some(e=>e.facets.some(e=>e.facetType===`creator`)),_=h.length===0?this.feedbackWidgetTemplate:n,v=s.length>0&&d.length>this.carouselMinTiles&&!g,y=c.length>0&&f.length>this.carouselMinTiles,b=m(i.slice(0,2),t=>{let r=e[t];return r.length>this.carouselMinTiles?this.smartCarouselSection({mediatype:t},`${C(X[t]??t)} items`,r.slice(0,this.carouselMaxTiles),{renderShowAllBtn:r.length>this.carouselMaxTiles}):n}),x=v?this.smartCarouselSection({creator:l},`By ${l}`,d.slice(0,this.carouselMaxTiles),{renderShowAllBtn:d.length>this.carouselMaxTiles}):n,S=y?this.smartCarouselSection({subject:u},`About ${u}`,f.slice(0,this.carouselMaxTiles),{renderShowAllBtn:f.length>this.carouselMaxTiles}):n;return a`<div id="tile-carousel-container" slot="smart-results">
      ${h}

      <h2 class="results-section-heading">
        ${o(`Text, captions, and transcripts`)}
        ${_}
      </h2>
      <federated-search-carousel
        .searchQuery=${this.searchQuery}
        .searchService=${this.searchService}
        .baseImageUrl=${this.baseImageUrl}
        .baseNavigationUrl=${this.baseNavigationUrl}
      ></federated-search-carousel>

      ${b} ${x} ${S}
    </div>`}get feedbackWidgetTemplate(){return a`<feature-feedback
      .buttonText=${o(`Feedback?`)}
      .featureIdentifier=${`SmartMultiCarousel`}
      .prompt=${o(`What do you think of the smart carousels?`)}
      .featureFeedbackService=${this.featureFeedbackService}
      .resizeObserver=${this.resizeObserver}
      .recaptchaManager=${this.recaptchaManager}
    ></feature-feedback>`}smartCarouselSection(e,t,r,i={renderShowAllBtn:!0,renderFeedbackWidget:!1}){return a`
      <h2 class="results-section-heading">
        ${t} ${i.renderFeedbackWidget?this.feedbackWidgetTemplate:n} ${i.renderShowAllBtn?a`<button
          class="results-show-all-btn"
          @click=${()=>this.showAllButtonClicked(e)}
        >
          ${o(`Show all...`)}
        </button>`:n}
      </h2>
      <tile-carousel
        enableHoverPanes
        .tiles=${r}
        .usePlaceholder=${this.loading}
        .baseImageUrl=${this.baseImageUrl}
        .baseNavigationUrl=${this.baseNavigationUrl}
      ></tile-carousel>
    `}showAllButtonClicked(e){this.dispatchEvent(new CustomEvent(`showAllClicked`,{detail:e}))}static get styles(){return e`
      .results-section-heading {
        margin: 0;
        padding-left: 9px;
        font-size: 2rem;
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
        float: right;
      }

      #tile-carousel-container {
        padding: 5px 0 8px;
        background-color: #eaf2fb;
        border: 1px solid var(--secondaryCTABorder, #999);
        border-radius: 10px;
      }

      feature-feedback {
        vertical-align: text-bottom;
      }
    `}};v([t({type:Object})],Z.prototype,`dataSource`,void 0),v([t({type:Object})],Z.prototype,`selectedFacets`,void 0),v([t({type:String})],Z.prototype,`searchQuery`,void 0),v([t({type:Object})],Z.prototype,`searchService`,void 0),v([t({type:Object})],Z.prototype,`featureFeedbackService`,void 0),v([t({type:Object})],Z.prototype,`recaptchaManager`,void 0),v([t({type:Object})],Z.prototype,`resizeObserver`,void 0),v([t({type:String})],Z.prototype,`baseImageUrl`,void 0),v([t({type:String})],Z.prototype,`baseNavigationUrl`,void 0),v([t({type:Number})],Z.prototype,`carouselMinTiles`,void 0),v([t({type:Number})],Z.prototype,`carouselMaxTiles`,void 0),v([t({type:Boolean})],Z.prototype,`loading`,void 0),v([s()],Z.prototype,`heuristicResults`,void 0),Z=v([g(`smart-carousel-group`),i()],Z);var Q=class extends c{render(){return this.articleData?.title?a`
      <article class="container">
        ${this.closeButtonTemplate} ${this.thumbnailTemplate}
        ${this.headerTemplate} ${this.infoboxTemplate}
      </article>
    `:n}get closeButtonTemplate(){let e=o(`Close Wikipedia section`);return a`
      <button
        class="close-button"
        title=${e}
        aria-label=${e}
        @click=${this.closeButtonClicked}
      >
        <img src=${w} aria-hidden="true" />
      </button>
    `}get thumbnailTemplate(){if(!this.articleData?.thumbnail)return n;let{href:e,src:t,alt:r}=this.articleData.thumbnail,i=a`
      <div class="thumbnail-frame">
        <img src=${t} alt=${h(r)} />
      </div>
    `;return a`
      <div class="thumbnail-col">
        ${e?a`<a href=${e}>${i}</a>`:i}
      </div>
    `}get headerTemplate(){if(!this.articleData)return n;let{title:e,url:t}=this.articleData;return a`
      <header class="header-col">
        <p class="wikipedia-credit">
          ${o(a`From <a href=${S}>Wikipedia</a>`)}
        </p>
        <h2 class="title">${e}</h2>
        <p class="extract">
          ${d(this.articleData.extract?.preparedHTML)}
          <a class="read-more" target="_blank" href=${h(t)}
            >${o(`Read More`)}</a
          >
        </p>
      </header>
    `}get infoboxTemplate(){if(!this.articleData?.infobox)return n;let{infobox:e}=this.articleData;return a`
      <div class="infobox-col">
        <table class="infobox">
          <tbody>
            ${m(e.entries,([e,t])=>a`
                <tr>
                  <th scope="row">${e}</th>
                  <td>${d(t)}</td>
                </tr>
              `)}
          </tbody>
        </table>
      </div>
    `}closeButtonClicked(){this.articleData=void 0,this.dispatchEvent(new CustomEvent(`close`))}static get styles(){return e`
      .container {
        position: relative;
        display: flex;
        padding: 30px 10px;
      }

      .close-button {
        position: absolute;
        top: 10px;
        right: 10px;
        width: 20px;
        height: 20px;
        padding: 0;
        border: none;
        background: none;
        appearance: none;
        cursor: pointer;
      }

      .thumbnail-col,
      .header-col,
      .infobox-col {
        padding: 0 15px;
      }

      .thumbnail-col {
        flex-basis: calc(100% / 6);
        text-align: center;
      }

      .header-col {
        flex-basis: 50%;
      }

      .infobox-col {
        flex-basis: calc(100% / 3);
      }

      .thumbnail-frame {
        display: inline-block;
        line-height: 0;
      }

      .thumbnail-frame > img {
        max-width: 180px;
        max-height: 180px;
      }

      .wikipedia-credit {
        margin: 0 0 5px;
        font-size: 1.2rem;
        line-height: 2rem;
      }

      .title {
        margin: 0 0 10px;
        font-size: 1.8rem;
        font-weight: 700;
        line-height: 2rem;
      }

      .extract {
        margin: 0 0 10px;
        font-size: 1.4rem;
      }

      .read-more {
        display: inline-block;
      }

      .infobox {
        font-size: 1.4rem;
        text-align: left;
        border-spacing: 2px 8px;
      }

      .infobox tr {
        padding-top: 1rem;
      }

      .infobox th {
        padding-right: 1rem;
        vertical-align: top;
      }

      .infobox td {
        display: -webkit-box;
        overflow: hidden;
        -webkit-box-orient: vertical;
        -webkit-line-clamp: 4;
      }

      .infobox a:link {
        display: inline-block;
      }

      .read-more::after,
      .infobox a:link[rel*='external']::after {
        display: inline-block;
        width: 1.2rem;
        height: 1.2rem;
        margin-left: 5px;
        transform: translateY(1px);
        filter: invert(32%) sepia(95%) saturate(2584%) hue-rotate(224deg)
          brightness(102%) contrast(101%); /* Matches #4b64ff */
        /* Don't let prettier convert the following quotes from " to '. They MUST be double-quotes. */
        /* prettier-ignore */
        content: url("${u(N)}");
      }

      a:link {
        color: var(--ia-theme-link-color, #4b64ff);
        text-decoration: none;
      }

      a:link:hover {
        text-decoration: underline;
      }

      @media (max-width: 991px) {
        .container {
          flex-direction: column;
        }

        .infobox-col {
          display: none;
        }
      }
    `}};v([t({type:Object})],Q.prototype,`articleData`,void 0),Q=v([g(`wikipedia-creator-block`),i()],Q);var ee=`data:image/svg+xml,%3csvg%20height='100'%20viewBox='0%200%20100%20100'%20width='100'%20xmlns='http://www.w3.org/2000/svg'%3e%3cg%20fill='%23000'%3e%3cpath%20d='m34.3023248%200c2.3024872%200%204.1976752%207.38057743%204.1976752%2016.3954506v11.191601c0%209.0183727-1.895188%2016.4129484-4.1976752%2016.4129484h-7.6067277c-2.3056042%200-4.1955971-7.3945757-4.1955971-16.4129484v-11.191601c0-9.01487317%201.8899929-16.3954506%204.1955971-16.3954506zm-17.7871275%2014.9998517c-.0100619.4602164-.0151973.9256873-.0151973%201.3955989v11.191601c0%20.476083.0052671.9476408.0155843%201.4138244l-11.46507925-.000876c-2.78931692%200-5.05050505-3.1340068-5.05050505-7s2.26118813-7%205.05050505-7zm27.9695649%200%2050.4647327.0001483c2.789317%200%205.0505051%203.1340068%205.0505051%207s-2.2611881%207-5.0505051%207l-50.4651207.000876c.0103443-.4661836.0156258-.9377414.0156258-1.4138244v-11.191601c0-.4699116-.0051495-.9353825-.0152378-1.3955989z'/%3e%3cpath%20d='m77.3023248%2056c2.3024872%200%204.1976752%207.3805774%204.1976752%2016.3954506v11.191601c0%209.0183727-1.895188%2016.4129484-4.1976752%2016.4129484h-7.6067277c-2.3056042%200-4.1955971-7.3945757-4.1955971-16.4129484v-11.191601c0-9.0148732%201.8899929-16.3954506%204.1955971-16.3954506zm-17.7871056%2014.9988505c-.0100764.4605425-.0152192.9263478-.0152192%201.3966001v11.191601c0%20.475743.0052595.9469672.0155622%201.4128254l-54.46505715.000123c-2.78931692%200-5.05050505-3.1340068-5.05050505-7s2.26118813-7%205.05050505-7zm27.9695211%200%207.4647546.0011495c2.789317%200%205.0505051%203.1340068%205.0505051%207s-2.2611881%207-5.0505051%207l-7.4650985-.000123c.0103296-.4658582.0156036-.9370824.0156036-1.4128254v-11.191601c0-.4702523-.005157-.9360576-.0152597-1.3966001z'/%3e%3c/g%3e%3c/svg%3e`,$=class extends c{constructor(...e){super(...e),this.selectedSort=E.default,this.sortDirection=null,this.facetLoadStrategy=`opt-in-or-login`,this.allowSmartResults=!1,this.smartCarouselType=`none`,this.showSmartFacetBar=!1,this.showMediatypeTabs=!1,this.showDiscoveryTab=!1,this.isTVSearch=!1,this.useMobileView=!1,this.searchType=y.DEFAULT,this.initialResultsPageLoading=!1,this.hasError=!1,this.federatedResultsLoading=!1,this.tabHasResults=Object.fromEntries(L.map(e=>[e,!0])),this.isManageView=!1,this.activeTabId=`all`,this.pendingCategoryId=null,this.tabState={},this.carouselModels={},this.isRedirecting=!1,this.redirectMessage=``,this.facetPaneVisible=!0,this.showMorePanelOpen=!1,this.showMoreDisplayType=`carousel`,this.wikipediaDataState=`unfetched`,this.analyticsNamespace=`Search`,this._tileCarouselSearches=new Map,this._tileCarouselPromises=new Map,this.onPageShow=e=>{e.persisted&&(this.isRedirecting=!1,this.redirectMessage=``)}}get externalSearchHandlers(){return{web:this.waybackSearchHandler}}get tabs(){return Object.entries(this.tabHasResults).filter(([e,t])=>t||e===this.activeTabId).filter(([e])=>e!==`discovery`||this.showDiscoveryTab).map(([e])=>{let t=e,n=F()[t];return{id:t,displayName:n.tabLabel,url:n.isRedirect&&this.externalSearchHandlers[t]?this.externalSearchHandlers[t].getSearchUrl(this.searchQuery):void 0}})}get searchBarCategories(){return this.tabs.map(e=>{let t=F()[e.id],n=this.useMobileView&&t.shortLabel?t.shortLabel:e.displayName;return{id:e.id,label:n}})}render(){return a`
      <div id="page-container" class=${f({tabbed:this.showMediatypeTabs,mobile:this.useMobileView})}>
        <div id="search-bar-container">
          <div
            id="search-bar-content-container"
            .class="${this.useMobileView?`mobile`:n}"
          >
            ${this.useMobileView?n:a`
                  <div id="search-name-container">
                    <h1>${o(`Search`)}</h1>
                  </div>
                `}
            <div id="search-bar-and-buttons">
              <ia-dropdown-search-bar
                .query=${this.searchQuery}
                .categories=${this.searchBarCategories}
                .selectedCategory=${this.pendingCategoryId??this.activeTabId}
                .placeholder=${this.isTVSearch?o(`Search TV News Archive captions`):o(`Search`)}
                ?hideDropdown=${this.isTVSearch}
                ?loading=${this.initialResultsPageLoading}
                @searchRequested=${this.searchRequested}
                @categoryChanged=${this.handleSearchBarCategoryChanged}
              ></ia-dropdown-search-bar>
              <div id="search-buttons">${this.searchButtonTemplates}</div>
            </div>
            ${this.useMobileView?n:this.actionBarTemplate}
          </div>
        </div>

        ${this.showMediatypeTabs?a`<tab-manager
              class=${f({"tabs-hidden":!this.shouldDisplayTabs})}
              .tabs=${this.tabs}
              .activeTabId=${this.activeTabId}
              .defaultTabId=${`all`}
              @tabChanged=${this.handleTabChanged}
              @linkTabClicked=${this.handleLinkTabClicked}
            >
              ${this.tabContentTemplate}
            </tab-manager>`:this.collectionBrowserTemplate}
      </div>
    `}get activeTabConfig(){if(this.showMediatypeTabs)return F()[this.activeTabId]}get shouldDisplayTabs(){return this.searchQuery?!!(this.initialResultsPageLoading||this.hasNonEmptyTab||this.hasAnyResults||this.showMediatypeTabs):!1}get hasNonEmptyTab(){return Object.values(this.tabState).some(({dataSource:e})=>e.size>0)}get hasTabInURL(){return new URL(window.location.href).searchParams.has(`tab`)}get hasAnyMetadataResults(){return this.showMediatypeTabs?Object.values(F()).filter(e=>e.searchType===y.DEFAULT).some(e=>this.tabState[e.id]?.dataSource.size>0):!1}get hasAnyFederatedResults(){return!!Object.values(this.federatedResults??{}).find(e=>e.length>0)}get hasAnyResults(){return this.hasAnyMetadataResults||this.hasAnyFederatedResults}hasFederatedResultsFor(...e){return e.some(e=>!!this.federatedResults?.[e]?.length)}get advancedSearchTemplate(){let{searchQuery:e,baseNavigationUrl:t}=this;return t===void 0?n:a`<a
      id="advanced-search-link"
      href=${`${t}/advancedsearch.php${e?`?q=${encodeURIComponent(e)}`:``}`}
      @click=${this.advancedSearchClicked}
    >
      ${o(`Advanced Search`)}
    </a>`}get cbSessionId(){try{return sessionStorage?.getItem(`cb-session`)??``}catch{return``}}get feedbackSurveyTemplate(){return a`
      <ia-feedback-survey
        showButtonThumbs
        showQuestionNumbers
        surveyIdentifier="Search Page (post-beta)"
        buttonText=${o(`Feedback`)}
        .featureFeedbackService=${this.featureFeedbackService}
        .recaptchaManager=${this.recaptchaManager}
        .resizeObserver=${this.resizeObserver}
      >
        <ia-survey-extra
          name="Session"
          .value=${this.cbSessionId}
        ></ia-survey-extra>
        <ia-survey-extra
          name="Query"
          .value=${this.searchQuery}
        ></ia-survey-extra>
        <!-- The survey stores each answer under its prompt text, so the
          prompts stay in English to keep answers in one bucket. -->
        <ia-survey-vote
          prompt="Did you find what you were searching for?"
          showComments
        ></ia-survey-vote>
        <ia-survey-comment
          prompt="Any other feedback about the search page?"
        ></ia-survey-comment>
        <span>
          ${o(`Responses are anonymous and are used for improving Internet Archive search interactions.`)}
        </span>
      </ia-feedback-survey>
    `}get searchButtonTemplates(){let e=[this.showMediatypeTabs?this.feedbackSurveyTemplate:n,this.advancedSearchTemplate],t=[...e.slice().reverse(),this.actionBarTemplate];return this.useMobileView?t:e}get showMorePanelSections(){if(!this.federatedResults)return[];let e=this.activeTabConfig;if(e?.showMore?.type!==`list`)return[];let t=e.showMore?.sections;return t?t.map(t=>({...t,tiles:this.federatedResults?.[t.federatedResultsKey].slice(0,14).map(e=>new T(e)).filter(t=>!e.isMediatype||t.mediatype===this.activeTabId)??[]})).filter(e=>e.tiles.length>0):[]}get tabContentTemplate(){if(this.isRedirecting)return a`<p class="redirect-message" slot=${this.activeTabId}>
        ${this.redirectMessage}
      </p>`;let e=a`
      <ia-icon .src=${M} aria-hidden="true"></ia-icon>
    `,t=this.showMorePanelOpen&&!this.activeTabConfig?.isDefaultShowMore?a`
          <button
            class="exit-search-inside-btn"
            @click=${this.toggleShowMorePanel}
          >
            ${e} ${o(`Back`)}
          </button>
        `:n;return a`
      <div class="tab-content-container" slot=${this.activeTabId}>
        ${this.wikipediaCreatorBlockTemplate} ${t}
        ${this.showMorePanelContent} ${this.collectionBrowserTemplate}
      </div>
    `}get showMorePanelContent(){if(!this.showMorePanelOpen||this.activeTabConfig?.showMore?.type===`full-search`)return n;if(this.federatedResultsLoading)return this.placeholderGridTemplate;this.activeTabId===`discovery`&&this.populateFederatedResults();let{showMorePanelSections:e}=this;return a`
      ${m(e,({target:e,heading:t,layoutType:n,tiles:r})=>a`
          <div class="tile-carousel-container">
            <h2 class="results-section-heading">
              ${t}
              <button
                class="carousel-show-more-btn"
                @click=${()=>{this.activeTabId=e.tab,this.showMorePanelOpen=e.isShowMore,window.scrollTo(0,0),this.updateCollectionBrowserForActiveTab()}}
              >
                ${o(`Show More...`)}
              </button>
            </h2>
            <tile-carousel
              class="${this.showMoreDisplayType} wide"
              enableHoverPanes
              .tiles=${r}
              .layoutType=${n}
              .baseNavigationUrl=${this.baseNavigationUrl}
              .baseImageUrl=${this.baseImageUrl}
              .resizeObserver=${this.resizeObserver}
              .loggedIn=${!!this.loggedInUser?.itemname}
              .analyticsHandler=${this.analyticsHandler}
              @tileClick=${()=>this.discoveryTileClicked(t)}
            ></tile-carousel>
          </div>
        `)}
    `}get placeholderGridTemplate(){return a`
      <tile-carousel
        class="grid wide"
        usePlaceholder
        .numPlaceholders=${50}
      ></tile-carousel>
    `}get collectionBrowserTemplate(){let{activeTabConfig:e,hasError:t,currentTabState:i,searchService:s}=this,c=e?.showMore,u=c?.type===`full-search`,d=this.showMediatypeTabs?i?.dataSource.totalResults??0:0,f;if(this.showMediatypeTabs){let t=this.showMorePanelOpen&&u?c.heading:e?.resultsHeading,n=this.initialResultsPageLoading?o(`Searching...`):o(r`${b(d)} Results`);f=a`
        <div slot="results-heading">
          ${l(this.shouldShowFiltersToggle,()=>this.filtersToggleTemplate)}
          <span class="results-category">${t}</span>
          <span class="results-count">${n}</span>
        </div>
      `}let p=this.showMorePanelOpen&&u?c.searchType:e?.searchType??n,m=this.isTVSearch?y.TV:p,h=!!this.searchQuery,g=this.collectionBrowser?.dataSource,_=g?.queryInitialized&&!this.initialResultsPageLoading&&g.size===0?a`<div class="search-empty-message" slot="cb-results">
          ${e?.emptyMessage??z()}
        </div>`:n,v=this.initialResultsPageLoading||this.hasAnyMetadataResults,x=c?.type===`list`&&this.showMorePanelOpen,S=this.showMediatypeTabs&&h&&!t,C=this.showMediatypeTabs&&!e?.allowsMediatypeFacets,w=this.showMorePanelOpen?f:[f,_];return a`
      ${this.showMorePanelOpen?n:this.carouselSectionTemplate}
      <div
        id="collection-browser-container"
        class=${this.showMorePanelOpen?`search-inside`:n}
        slot=${this.showMediatypeTabs?this.activeTabId:n}
        ?hidden=${x}
      >
        <collection-browser
          class=${this.showMediatypeTabs?`tabbed`:n}
          .selectedSort=${this.selectedSort}
          .sortDirection=${this.sortDirection}
          .baseNavigationUrl=${this.baseNavigationUrl}
          .baseImageUrl=${this.baseImageUrl}
          .searchService=${s}
          .searchType=${m}
          .resizeObserver=${this.resizeObserver}
          .pageContext=${`search`}
          .showHistogramDatePicker=${!0}
          .showSmartFacetBar=${this.showSmartFacetBar&&v}
          .smartFacetBarLabel=${o(`Browse:`)}
          .facetPaneVisible=${this.facetPaneVisible}
          .facetLoadStrategy=${this.facetLoadStrategy}
          .loggedIn=${!!this.loggedInUser?.itemname}
          .isManageView=${this.isManageView}
          .analyticsHandler=${this.analyticsHandler}
          .modalManager=${this.modalManager}
          .featureFeedbackService=${this.featureFeedbackService}
          .recaptchaManager=${this.recaptchaManager}
          .searchContext=${this.analyticsNamespace}
          ?suppressMediatypeFacets=${C}
          ?suppressResultCount=${this.showMediatypeTabs}
          ?suppressPlaceholders=${S}
          ?suppressURLSinParam=${this.showMediatypeTabs||this.isTVSearch}
          @baseQueryChanged=${this.handleBaseQueryChanged}
          @queryStateChanged=${this.handleQueryStateChanged}
          @searchResultsLoadingChanged=${this.handleSearchResultsLoadingChanged}
          @facetPaneVisibilityChanged=${this.handleFacetPaneVisibilityChanged}
          @searchTypeChanged=${this.searchTypeChanged}
          @searchError=${this.handleSearchError}
          @itemRemovalRequested=${this.handleItemRemovalRequested}
          @itemManagerRequested=${this.handleItemManagerRequested}
          @manageModeChanged=${this.updateManageParam}
        >
          ${w}
        </collection-browser>
      </div>
    `}get shouldShowFiltersToggle(){return this.showMediatypeTabs&&this.shouldDisplayTabs&&!this.useMobileView}get filtersToggleTemplate(){let e=a`<img src=${ee} alt="" />`;return a`
      <button
        id="filters-toggle"
        class=${this.facetPaneVisible?`active`:``}
        title="${this.facetPaneVisible?`Hide`:`Show`} filters pane"
        @click=${this.filtersToggleClicked}
      >
        ${e}
      </button>
    `}filtersToggleClicked(){this.facetPaneVisible=!this.facetPaneVisible,this.sendAnalyticsEvent({action:this.facetPaneVisible?`FiltersToggledOpen`:`FiltersToggledClosed`,label:`tab:${this.activeTabId}`})}discoveryTileClicked(e){this.sendAnalyticsEvent({action:`DiscoveryTileClicked`,label:`section:${e}`})}get wikipediaCreatorBlockTemplate(){return this.wikipediaDataState===`has-data`?a`
      <div id="wikipedia-container">
        <wikipedia-creator-block
          .articleData=${this.wikipediaCreatorData}
          @close=${this.handleWikipediaSectionClosed}
        ></wikipedia-creator-block>
      </div>
    `:n}get actionBarTemplate(){return a`
      <div id="action-bar-spacing">
        <action-bar
          .appServices=${this.appServices}
          .url=${this.shareableUrl}
          .title=${this.searchQuery??``}
          .analyticsCategory=${this.analyticsNamespace}
          .mediaType=${`search`}
          .identifier=${this.searchQuery}
          .titleSearch=${this.currentBaseUrl}
          .loggedIn=${!!this.loggedInUser}
          ?showManageButton=${this.userHasManagePrivs}
          @removeItemsPressed=${this.removeItemsPressed}
        ></action-bar>
      </div>
    `}get shareableUrl(){let e=new URL(window.location.href);return[...e.searchParams.entries()].forEach(([t,n])=>{n||e.searchParams.delete(t)}),e.toString()}get carouselSectionTemplate(){if(!this.showMediatypeTabs&&this.smartCarouselType===`none`)return n;switch(this.smartCarouselType){case`multi`:return this.multiCarouselSectionTemplate;case`single`:return this.singleCarouselSectionTemplate({config:{heading:`Text, Captions, and Transcripts`,layoutType:`snippets-only`,source:{type:`federated`,keys:[`fts`,`rcs`,`whisper`,`tvs`],tilesPerSource:3}}});case`single-tabbed`:return this.singleCarouselSectionTemplate({renderShowMoreBtn:!0});default:return n}}singleCarouselSectionTemplate(e={}){let t=e.renderShowMoreBtn?a`<button
          class="carousel-show-more-btn"
          @click=${this.toggleShowMorePanel}
        >
          ${o(`Show More...`)}
        </button>`:n,{activeTabConfig:r}=this,i=e.config??r?.carousel;if(!i)return n;let s=e=>Array.isArray(e)&&e.length===0,c=i.source.type===`federated`&&(!this.federatedResults||this.hasFederatedResultsFor(...i.source.keys)),l=i.source.type===`single-service`&&!s(this.carouselModels[i.source.searchType]);return!c&&!l?n:a`<section
      class="tile-carousel-container"
      slot="smart-results"
      aria-labelledby="highlights-heading"
    >
      <div class="tile-carousel-top-row">
        <h2 id="highlights-heading" class="results-section-heading">
          ${i.heading}
        </h2>
        ${t}
      </div>
      ${this.getCarouselTemplate(i)}
    </section>`}getCarouselTemplate(e){let{source:t,layoutType:r}=e;if(t.type===`single-service`){let{tiles:e,settled:n}=this.tileCarouselPromises(t);return a`
        <tile-carousel
          class="wide"
          enableHoverPanes
          .baseImageUrl=${this.baseImageUrl}
          .baseNavigationUrl=${this.baseNavigationUrl}
          .layoutType=${r??`default`}
          .tiles=${p(e,[])}
          ?usePlaceholder=${p(n,!0)}
          @tileClick=${this.carouselTileClicked}
        ></tile-carousel>
      `}let{tilesPerSource:i,keys:o}=e.source;return a`
      <federated-search-carousel
        class=${this.facetPaneVisible?``:`wide`}
        .searchQuery=${this.searchQuery}
        .searchService=${this.searchService}
        .baseImageUrl=${this.baseImageUrl}
        .baseNavigationUrl=${this.baseNavigationUrl}
        .maxTiles=${20}
        .tilesPerService=${i}
        .serviceKeys=${o}
        .mediatypeFilter=${this.activeTabConfig?.isMediatype?[this.activeTabId]:n}
        .size=${this.facetPaneVisible?`normal`:`wide`}
        .layoutType=${r??`default`}
        .federatedResults=${this.federatedResults}
        ?tilesLoading=${this.federatedResultsLoading}
      ></federated-search-carousel>
    `}tileCarouselPromises(e){let t=this._tileCarouselPromises.get(e.searchType);if(t)return t;let n=this.populateTileCarouselResults(e),r={tiles:n,settled:n.then(()=>!1)};return this.searchService&&this._tileCarouselPromises.set(e.searchType,r),r}async populateTileCarouselResults(e){let{searchType:t,numTiles:n}=e,r=this._tileCarouselSearches.get(t);if(r)return r;let i=this.carouselModels[t];if(i)return i;if(!this.searchService)return[];let a=this.searchService?.search({query:this.searchQuery,aggregations:{omit:!0},rows:100,page:1},t).then(e=>{let r=e.success?e.success.response.results.slice(0,n).map(e=>new T(e,`search_query`)):[];return this._tileCarouselSearches.get(t)===a&&(this._tileCarouselSearches.delete(t),this.carouselModels[t]=r),r});return this._tileCarouselSearches.set(t,a),a}carouselTileClicked(){this.sendAnalyticsEvent({action:`CarouselTileClicked`,label:`tab:${this.activeTabId}`})}toggleShowMorePanel(){this.showMorePanelOpen=!this.showMorePanelOpen,this.updateCollectionBrowserForActiveTab(),this.sendAnalyticsEvent({action:this.showMorePanelOpen?`ShowMoreOpened`:`ShowMoreClosed`,label:`tab:${this.activeTabId}`})}get multiCarouselSectionTemplate(){return this.isFacetFiltered?this.singleCarouselSectionTemplate():a`<div slot="smart-results">
      <smart-carousel-group
        .dataSource=${this.collectionBrowser?.dataSource}
        .selectedFacets=${this.collectionBrowser?.selectedFacets}
        .searchQuery=${this.searchQuery}
        .searchService=${this.searchService}
        .featureFeedbackService=${this.featureFeedbackService}
        .recaptchaManager=${this.recaptchaManager}
        .resizeObserver=${this.resizeObserver}
        .baseImageUrl=${this.baseImageUrl}
        .baseNavigationUrl=${this.baseNavigationUrl}
        ?loading=${this.initialResultsPageLoading}
        @showAllClicked=${this.carouselShowAllClicked}
      ></smart-carousel-group>
    </div>`}carouselShowAllClicked(e){this.collectionBrowser.selectedFacets||(this.collectionBrowser.selectedFacets=O());for(let[t,n]of Object.entries(e.detail))this.collectionBrowser.selectedFacets={...this.collectionBrowser.selectedFacets,[t]:{...this.collectionBrowser.selectedFacets[t],[n]:{key:n,state:`selected`,count:0}}}}handleTabChanged(e){if(!R(e.detail)){x(`Unknown tab selection:`,e.detail);return}let t=e.detail,n=F()[t];this.sendAnalyticsEvent({action:`TabChanged`,label:t});let{isRedirect:r,redirectMessage:i}=n,a=this.externalSearchHandlers[t];if(r&&a){a.performSearch(this.searchQuery),this.isRedirecting=!0,this.redirectMessage=i??I();return}this.applyPendingCategory(t),this.updateCollectionBrowserForActiveTab()}handleSearchBarCategoryChanged(e){if(!R(e.detail)){x(`Unknown category selection:`,e.detail);return}let t=e.detail;this.sendAnalyticsEvent({action:`SearchCategoryChanged`,label:t}),this.pendingCategoryId=t}handleLinkTabClicked(e){if(!R(e.detail)){x(`Unknown link tab click:`,e.detail);return}let t=e.detail,n=F()[t];this.sendAnalyticsEvent({action:`LinkTabClicked`,label:t});let{isRedirect:r,redirectMessage:i}=n;if(r){this.isRedirecting=!0,this.redirectMessage=i??I();return}}handleFederatedResultsLoaded(){this.updateTabVisibility()}updateTabVisibility(){let{federatedResults:e}=this;if(!e){this.tabHasResults=Object.fromEntries(L.map(e=>[e,!0]));return}this.tabHasResults={all:!0,texts:e.texts.length>0||e.fts.length>0,fulltext:e.fts.length>0,radio:e.rcs.length>0,tv:e.tvs.length>0,movies:e.movies.length>0,audio:e.audio.length>0,software:e.software.length>0,image:e.image.length>0,etree:e.etree.length>0,collection:!0,data:!0,web:!0,discovery:!0}}async populateFederatedResults(){if(this.showMediatypeTabs&&this.activeTabId!==`all`&&this.searchQuery)return this.federatedResults?this.federatedResults:this._federatedResultsPromise?this._federatedResultsPromise:(this.federatedResultsLoading=!0,this._federatedResultsPromise=P(this.searchQuery,this.searchService),this.federatedResults=await this._federatedResultsPromise,this.federatedResultsLoading=!1,this._federatedResultsPromise=void 0,this.handleFederatedResultsLoaded(),this.federatedResults)}handleQueryStateChanged(e){this.selectedSort=this.collectionBrowser.selectedSort,this.sortDirection=this.collectionBrowser.sortDirection,this.activeTabId===`tv`&&this.selectedSort===E.default&&(this.selectedSort=E.datearchived,this.sortDirection=`desc`),this.showMediatypeTabs&&(this.tabState[this.fullTabStateKey]||(this.tabState[this.fullTabStateKey]=this.newTabState()),this.tabState[this.fullTabStateKey].queryState=e.detail,this.applyCollectionBrowserQuery(e.detail?.baseQuery??this.searchQuery),this.activeTabId===`all`&&(this.tabState[this.activeTabId].dataSource=this.collectionBrowser.dataSource))}handleSearchError(){this.hasError=!0}removeItemsPressed(){this.collectionBrowser&&(this.collectionBrowser.isManageView=!this.collectionBrowser.isManageView)}async setupInitialTab(){let e=new URL(location.href),{searchParams:t}=e,n=t.get(`tab`),r=t.get(`sin`);if(n&&R(n))this.activeTabId=n;else if(r){let n=B[r];n&&(this.activeTabId=n.tab,this.showMorePanelOpen=n.isShowMore,t.delete(`sin`),history.replaceState(null,``,e),await this.updateComplete,this.tabManager?.persistActiveTabToURL())}else this.activeTabId=`all`;this.tabHasResults={...this.tabHasResults,[this.activeTabId]:!0},this.updateCollectionBrowserForActiveTab()}get fullTabStateKey(){return this.activeTabId+(this.showMorePanelOpen?`-show-more`:``)}get currentTabState(){return this.tabState[this.fullTabStateKey]}newTabState(){let{activeTabConfig:e,searchQuery:t}=this,n={baseQuery:t,internalFilters:e?.filters??{},searchType:e?.searchType??y.DEFAULT,selectedSort:e?.defaultSortField??this.selectedSort,sortDirection:e?.defaultSortDirection??this.sortDirection,selectedTitleFilter:null,selectedCreatorFilter:null},r=e?.showMore;this.showMorePanelOpen&&r?.type===`full-search`&&(n.searchType=r.searchType,n.selectedSort=r.defaultSortField??this.selectedSort,n.sortDirection=r.defaultSortDirection??this.sortDirection);let i=new k(this.collectionBrowser);return this.hasManageParam&&i.setNumInitialPages(this.collectionBrowser.maxPagesToManage??15),{dataSource:i,queryState:n}}updateCollectionBrowserForActiveTab(){let{fullTabStateKey:e}=this;this.tabState[e]||(this.tabState[e]=this.newTabState()),this.installDataSource(),this.collectionBrowser.refreshVisibleResults(),this.collectionBrowser.refreshSmartFacets()}async installDataSource(){let{queryState:e,dataSource:t}=this.currentTabState;this.searchType=e.searchType,this.collectionBrowser&&(this.collectionBrowser.searchService=this.searchService,this.collectionBrowser.installDataSourceAndQueryState(t,e))}get isFacetFiltered(){return Object.values(this.collectionBrowser?.selectedFacets??{}).some(e=>Object.keys(e).length>0)}get hasManageParam(){return new URL(window.location.href).searchParams.get(`manage`)===`1`}get userHasManagePrivs(){let{privs:e}=this.loggedInUser??{};return(e?.includes(`/multi`)||e?.includes(`/`))??!1}updateManageParam(){let e=this.collectionBrowser?.isManageView??!1,t=new URL(window.location.href);e?t.searchParams.set(`manage`,`1`):t.searchParams.delete(`manage`),window.history.replaceState({},``,t)}handleBaseQueryChanged(e){this.applyCollectionBrowserQuery(e.detail.baseQuery)}applyCollectionBrowserQuery(e){e!==this.searchQuery&&(this.resetSearchState(),this.clearOtherTabStates(),this.searchQuery=e)}clearOtherTabStates(){let e=this.tabState[this.fullTabStateKey];this.tabState=e?{[this.fullTabStateKey]:e}:{}}handleSearchResultsLoadingChanged(e){this.initialResultsPageLoading=e.detail.loading,this.initialResultsPageLoading&&(this.hasError=!1),this.activeTabConfig?.carousel?.source.type===`federated`&&this.showMediatypeTabs&&!this.federatedResults&&!this.federatedResultsLoading&&this.initialResultsPageLoading&&this.populateFederatedResults()}handleFacetPaneVisibilityChanged(e){this.facetPaneVisible=e.detail}handleWikipediaSectionClosed(){this.wikipediaDataState=`closed`,this.wikipediaCreatorData=void 0}applyPendingCategory(e){this.activeTabId=e,this.searchType=F()[e]?.searchType??y.DEFAULT,this.showMorePanelOpen=!!F()[e]?.isDefaultShowMore,this.pendingCategoryId=null}resetSearchState(){this.carouselModels={},this.federatedResults=void 0,this._federatedResultsPromise=void 0,this._tileCarouselSearches.clear(),this._tileCarouselPromises.clear()}refreshSearchResults(){let e=this.fullTabStateKey;this.searchQuery||(this.showMorePanelOpen=!1);let t=this.activeTabConfig?.carousel?.source.type===`federated`;this.showMediatypeTabs&&this.searchQuery&&t&&this.populateFederatedResults(),this.collectionBrowser.searchType=this.searchType;let n=this.fullTabStateKey!==e;this.showMediatypeTabs&&n&&this.updateCollectionBrowserForActiveTab()}searchRequested(e){let{query:t,category:n}=e.detail,r=R(n)?F()[n]:void 0,i=this.externalSearchHandlers[n];if(r?.isRedirect&&i&&t){i.performSearch(t);return}let a=R(n)&&n!==this.activeTabId,o=this.searchQuery!==t,s=this.searchType!==this.collectionBrowser.searchType;!o&&!s&&!a||(this.searchQuery=t,this.clearOtherTabStates(),this.resetSearchState(),a&&this.tabManager?.selectTab(n),a&&this.activeTabId===n||(a&&this.applyPendingCategory(n),this.refreshSearchResults()),this.sendAnalyticsEvent({action:`SearchRequested`}))}searchTypeChanged(e){this.searchType=e.detail}advancedSearchClicked(){this.sendAnalyticsEvent({action:`AdvancedSearchClicked`})}async handleItemRemovalRequested(e){let t=e.detail.items;if(t?.length){let e=await(await this.appServices?.get())?.itemRemovalService.get();try{this.collectionBrowser.showRemoveItemsProcessingModal(),await e?.darkItems(t),this.collectionBrowser.removeCheckedTiles(),this.collectionBrowser.isManageView=!1,this.modalManager?.closeModal(),this.modalManager?.classList.remove(`remove-items`)}catch{this.collectionBrowser.showRemoveItemsErrorModal()}}}async handleItemManagerRequested(e){let t=e.detail.items;if(t?.length){let e=(await(await this.appServices?.get())?.appConfig.get())?.itemManagementUrl;e&&J({action:e,fields:{identifier:t.join(`,`)}})}}get queriedCreator(){let e=this.searchQuery?.match(/(?:^| )creator:(?:"([^"]+)"|\(([^)]+)\)|(\S+))/),t=e?.[1]??e?.[2]??e?.[3];return t?K(t):null}sendAnalyticsEvent(e){this.analyticsHandler?.sendEvent({category:this.analyticsNamespace,action:e.action,label:e.label})}connectedCallback(){super.connectedCallback(),window.addEventListener(`pageshow`,this.onPageShow)}disconnectedCallback(){super.disconnectedCallback(),window.removeEventListener(`pageshow`,this.onPageShow),this.resizeObserver&&this.disconnectResizeObserver(this.resizeObserver)}handleResize(e){e.target===this.searchBarContentContainer&&(this.useMobileView=e.contentRect.width<600)}willUpdate(){let e=new URL(window.location.href).searchParams;this.isTVSearch&&!e.has(`sort`)&&(this.selectedSort=E.datearchived,this.sortDirection=`desc`)}firstUpdated(){if(this.sendAnalyticsEvent({action:`PageView`}),!this.isTVSearch){let e=new URL(window.location.href).searchParams.get(`sin`);switch(e){case`TXT`:this.searchType=y.FULLTEXT;break;case`RADIO`:this.searchType=y.RADIO;break;default:this.searchType=y.DEFAULT}this.handleSinRedirect(e)}this.hasManageParam&&this.collectionBrowser.dataSource.setNumInitialPages(this.collectionBrowser.maxPagesToManage??15),this.smartCarouselType===`multi`&&this.collectionBrowser.dataSource.setPageSize(500),this.showMediatypeTabs&&this.setupInitialTab()}updated(e){if(e.has(`appServices`)&&this.appServices&&this.setupAppServicesDependents(),e.has(`searchQuery`)&&(this.setupPageMetadata(),this.queryUpdated()),e.has(`showSmartFacetBar`)&&(this.showSmartFacetBar||(this.facetPaneVisible=!0)),e.has(`resizeObserver`)){let t=e.get(`resizeObserver`);t&&this.disconnectResizeObserver(t),this.setupResizeObserver()}let t=this.shadowRoot?.getElementById(`wikipedia-container`);t&&requestAnimationFrame(()=>t.classList.add(`open`))}disconnectResizeObserver(e){e.removeObserver({target:this.searchBarContentContainer,handler:this})}setupResizeObserver(){this.resizeObserver&&this.resizeObserver.addObserver({target:this.searchBarContentContainer,handler:this})}async setupAppServicesDependents(){let e=await this.appServices?.get();if(!e)return;let[t,n,r,i,a,o,s,c,l,u,d]=await Promise.all([e.appConfig.get(),e.searchService.get(),e.resizeObserver.get(),e.waybackSearchHandler.get(),e.abtestManager.get(),e.featureFeedbackService.get(),e.recaptchaManager.get(),e.analyticsHandler.get(),e.modalManager.get(),e.pageMetadataService.get(),e.wikipediaAPIService.get()]);this.baseNavigationUrl=t?.archiveNavUrl,this.baseImageUrl=t?.baseImageUrl,this.currentBaseUrl=t?.currentBaseUrl,this.searchService=n,this.resizeObserver=r,this.waybackSearchHandler=i,this.abtestManager=a,this.featureFeedbackService=o,this.recaptchaManager=s,this.analyticsHandler=c,this.modalManager=l,this.pageMetadataService=u,this.wikipediaAPIService=d,await Promise.allSettled([this.setupDiscoveryTab(),this.setupLoggedInUser(),this.setupPageMetadata(),this.setupFacetsABTest(e)]),this.applySearchHandlers()}applySearchHandlers(){let e=this.externalSearchHandlers[this.activeTabId];e&&e.performSearch(this.searchQuery)}async setupLoggedInUser(){let e=await(await this.appServices?.get())?.userService.get();this.loggedInUser=(await e?.getLoggedInUser())?.success,this.setupManageView()}setupManageView(){this.hasManageParam&&this.userHasManagePrivs&&(this.collectionBrowser.isManageView=!0)}async setupPageMetadata(){this.searchQuery&&await Promise.all([this.pageMetadataService?.clearLinkTags(),this.pageMetadataService?.setMetaTag({name:`robots`,content:`noindex, nofollow`}),this.pageMetadataService?.addLink({rel:`canonical`,path:`/search?query=${this.searchQuery}`})])}async setupFacetsABTest(e){await(await e?.abtestManager.get())?.variantFor(`EagerFacets`)===`On`&&(this.facetLoadStrategy=`eager`)}setupDiscoveryTab(){new URL(location.href).searchParams.has(`discovery`)&&(this.showDiscoveryTab=!0)}queryUpdated(){this.collectionBrowser.baseQuery=this.searchQuery,this.updateWikipediaCreatorDataIfNeeded()}async updateWikipediaCreatorDataIfNeeded(){this.wikipediaCreatorData=void 0,this.wikipediaDataState=`unfetched`;let{queriedCreator:e}=this;if(e)try{this.wikipediaDataState=`fetching`;let t=await(await(await this.appServices?.get())?.wikipediaAPIService.get())?.fetchArticleData(e);t&&t.title?(this.wikipediaCreatorData=t,this.wikipediaDataState=`has-data`):this.wikipediaDataState=`no-data`}catch{this.wikipediaCreatorData=void 0,this.wikipediaDataState=`no-data`}}async handleSinRedirect(e){let t=await(await this.appServices?.get())?.waybackSearchHandler.get();switch(e){case`WEB`:t?.performSearch(this.searchQuery??``);return;default:}}static get styles(){return e`
      search-bar {
        height: 100px;
      }
      #page-container {
        font-size: 1.4rem;
        margin: 0 0 30px;
      }
      #page-container.tabbed {
        --tileBoxShadow: none;
        --tileHoverBoxShadow: 0 0 10px 0px rgba(8, 8, 32, 0.15);
      }

      .hide {
        display: none;
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
      }

      ia-feedback-survey {
        --featureFeedbackBetaButtonBackground: white;
      }

      ia-feedback-survey > span {
        display: block;
        margin: 0 10px 15px;
        font-size: 1.2rem;
        text-align: center;
      }

      ia-survey-vote {
        --commentHeight: 3.2rem;
      }

      ia-survey-vote,
      ia-survey-comment {
        --commentResize: vertical;
      }

      #highlights-heading {
        font-size: 1.6rem;
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

      #search-bar-and-buttons {
        flex: 1;
      }

      ia-dropdown-search-bar {
        --search-bar-width: 100%;
        --dropdownFontSize: 1.4rem;
        --dropdownItemButtonPadding: 0 10px;
      }

      #search-buttons {
        display: flex;
        column-gap: 5px;
        margin-top: 10px;
      }

      #advanced-search-link {
        margin-left: auto;
      }

      .mobile #advanced-search-link {
        margin-left: 0;
        margin-right: auto;
      }

      #action-bar-spacing {
        margin-left: 1rem;
        font-size: 1.3rem;
        height: fit-content;
      }

      .mobile #action-bar-spacing {
        --iconLabelGutterWidth: 4px;
        margin-top: -4px; /* Better icon alignment */
      }

      #wikipedia-container {
        max-height: 0;
        max-width: var(--app-page-max-width);
        margin: auto;
        overflow: hidden;
        transition: max-height 0.5s cubic-bezier(0.8, 0, 0.75, 1) 0s;
      }
      #wikipedia-container.open {
        max-height: 500px;
      }

      tab-manager {
        --tab-manager-tabs-row-padding-top: 0;
        --tab-manager-tabs-row-height: 34px;
        --tab-manager-tab-padding: 6px 5px;
        --tab-manager-tab-font-size: 1.4rem;
      }

      #filters-toggle {
        margin: 0 5px 0 0;
        border: 0;
        padding: 5px 8px;
        max-height: fit-content;
        border-radius: 5px;
        background: white;
        color: #2c2c2c;
        border: 1px solid #194880;
        font-size: 1.4rem;
        font-family: inherit;
        text-decoration: none;
        cursor: pointer;
        vertical-align: 2px;
      }

      #filters-toggle.active {
        background: #194880;
        color: white;
      }

      #filters-toggle > img {
        width: 12px;
        filter: invert(0.16667);
        vertical-align: -1px;
      }

      #filters-toggle.active > img {
        filter: invert(1);
      }

      #collection-browser-container {
        padding-left: 0.5rem;
        margin-bottom: 2rem;
      }

      #collection-browser-container.search-inside {
        padding: 5px;
        background-color: #eaf2fb;
        border: 1px solid var(--secondaryCTABorder, #999);
        border-radius: 10px;
      }

      collection-browser {
        max-width: var(--app-page-max-width);
        margin: auto;
      }

      collection-browser.tabbed {
        --facetsContainerMarginTop: 0;
        --rightColumnPaddingTop: 10px;
      }

      .results-section-heading {
        margin: 0;
        padding-left: 12px;
        font-size: 2rem;
        line-height: 25px;
      }
      .results-section-heading:not(:first-child) {
        margin-top: 10px;
      }

      .results-category {
        margin-right: 5px;
      }

      .results-count {
        font-size: 1.8rem;
        font-weight: normal;
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
        padding: 10px 0 6px;
        background-color: #eaf2fb;
        border: 1px solid var(--secondaryCTABorder, #999);
        border-radius: 10px;
      }

      .tile-carousel-top-row {
        display: flex;
        align-items: center;
        column-gap: 5px;
      }

      tile-carousel,
      federated-search-carousel {
        --imgBlockHeight: 8rem;
      }

      .carousel-show-more-btn,
      .exit-search-inside-btn {
        appearance: none;
        color: var(--ia-theme-link-color);
        background: transparent;
        border: 0;
        cursor: pointer;
        font-size: 15px;
        padding: 0;
        text-decoration: none;
      }
      .carousel-show-more-btn {
        margin-left: 5px;
      }
      .exit-search-inside-btn {
        padding: 10px 5px;
        display: inline-flex;
        align-items: center;
        column-gap: 5px;
      }

      .exit-search-inside-btn > ia-icon {
        display: inline-block;
        --iconHeight: 1.4rem;
      }

      iaux-toggle-switch {
        --labelFontSize: 1.4rem;
        margin-left: 10px;
      }

      .tab-content-container > .tile-carousel-container {
        margin-bottom: 15px;
      }

      .beta-badge {
        font-size: 12px;
        font-weight: bold;
        font-style: italic;
        color: #194880;
        border: 1px solid #194880;
        border-radius: 4px;
        padding: 1px 5px;
        margin-left: 5px;
        vertical-align: bottom;
      }

      .redirect-message {
        margin: 2rem 0;
        text-align: center;
        font-weight: bold;
      }

      .search-empty-message {
        font-size: 1.4rem;
        font-weight: bold;
        text-align: center;
        margin: 20px 0;
      }
    `}};v([t({type:String})],$.prototype,`searchQuery`,void 0),v([t({type:String})],$.prototype,`selectedSort`,void 0),v([t({type:String})],$.prototype,`sortDirection`,void 0),v([t({type:Object})],$.prototype,`appServices`,void 0),v([t({type:String})],$.prototype,`facetLoadStrategy`,void 0),v([t({type:Boolean})],$.prototype,`allowSmartResults`,void 0),v([t({type:String})],$.prototype,`smartCarouselType`,void 0),v([t({type:Boolean})],$.prototype,`showSmartFacetBar`,void 0),v([t({type:Boolean})],$.prototype,`showMediatypeTabs`,void 0),v([t({type:Boolean})],$.prototype,`showDiscoveryTab`,void 0),v([t({type:Boolean})],$.prototype,`isTVSearch`,void 0),v([s()],$.prototype,`searchService`,void 0),v([s()],$.prototype,`resizeObserver`,void 0),v([s()],$.prototype,`waybackSearchHandler`,void 0),v([s()],$.prototype,`abtestManager`,void 0),v([s()],$.prototype,`featureFeedbackService`,void 0),v([s()],$.prototype,`recaptchaManager`,void 0),v([s()],$.prototype,`wikipediaAPIService`,void 0),v([s()],$.prototype,`baseNavigationUrl`,void 0),v([s()],$.prototype,`baseImageUrl`,void 0),v([s()],$.prototype,`currentBaseUrl`,void 0),v([s()],$.prototype,`useMobileView`,void 0),v([s()],$.prototype,`loggedInUser`,void 0),v([s()],$.prototype,`searchType`,void 0),v([s()],$.prototype,`initialResultsPageLoading`,void 0),v([s()],$.prototype,`hasError`,void 0),v([s()],$.prototype,`wikipediaCreatorData`,void 0),v([s()],$.prototype,`federatedResults`,void 0),v([s()],$.prototype,`federatedResultsLoading`,void 0),v([s()],$.prototype,`tabHasResults`,void 0),v([s()],$.prototype,`isManageView`,void 0),v([s()],$.prototype,`activeTabId`,void 0),v([s()],$.prototype,`pendingCategoryId`,void 0),v([s()],$.prototype,`tabState`,void 0),v([s()],$.prototype,`carouselModels`,void 0),v([s()],$.prototype,`isRedirecting`,void 0),v([s()],$.prototype,`redirectMessage`,void 0),v([s()],$.prototype,`facetPaneVisible`,void 0),v([s()],$.prototype,`showMorePanelOpen`,void 0),v([s()],$.prototype,`showMoreDisplayType`,void 0),v([_(`collection-browser`)],$.prototype,`collectionBrowser`,void 0),v([_(`#search-bar-content-container`)],$.prototype,`searchBarContentContainer`,void 0),v([_(`tab-manager`)],$.prototype,`tabManager`,void 0),$=v([g(`search-page`),i()],$);var te=$;export{te as default};