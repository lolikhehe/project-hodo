const __vite__mapDeps=(i,m=__vite__mapDeps,d=(m.f||(m.f=["offshoot_assets/home-page-services-3ps3baAX.js","offshoot_assets/dist-naAOPxYm.js","offshoot_assets/vendor-shared-62wh2Xkx.js","offshoot_assets/dist-C2wBh3A_.js","offshoot_assets/no-news-posts-override-B2Yx8f-o.js"])))=>i.map(i=>d[i]);
import{A as e,C as t,E as n,F as r,N as i,O as a,P as o,S as s,T as c,v as l,w as u,x as d}from"./vendor-lit-MQ4DzmMc.js";import"./vendor-topnav-Dcg-INpV.js";import{_ as f,c as p,d as m,h,t as g,u as _}from"./index-NFRVgqV2.js";import"./ia-status-indicator-Bd61y7bd.js";import"./ia-dropdown-search-bar-D0po3r-G.js";import"./dist-CTMeFT4b.js";import{t as v}from"./no-news-posts-override-B2Yx8f-o.js";import"./dist-CD3AhsrO.js";import"./dist-Nfn4GWEx.js";import{t as y}from"./collection-BgJMB0Jo.js";import"./basic-carousel-DAFJS9qN.js";import{a as b,c as x,i as S,l as C,n as w,o as T,r as E,s as D,t as O}from"./web-md_No8fd.js";import{a as k}from"./models-DmFnvYaY.js";var A=class extends c{constructor(...e){super(...e),this.isLoading=!1}render(){let e=C();return a`
      <div id="mediacount-icon-container">
        ${this.generateIconTemplate({icon:O,title:e.web.title,svgFilter:e.web.svgFilter,link:`/details/web`,count:this.mediaCount?.web})}
        ${this.generateIconTemplate({icon:S,title:e.texts.title,svgFilter:e.texts.svgFilter,link:`/details/texts`,count:this.mediaCount?.texts})}
        ${this.generateIconTemplate({icon:w,title:e.movies.title,svgFilter:e.movies.svgFilter,link:`/details/movies`,count:this.mediaCount?.movies})}
        ${this.generateIconTemplate({icon:x,title:e.audio.title,svgFilter:e.audio.svgFilter,link:`/details/audio`,count:this.mediaCount?.audio})}
        ${this.generateIconTemplate({icon:E,title:e.tv.title,svgFilter:e.tv.svgFilter,link:`/details/tv`,count:this.mediaCount?.tv})}
        ${this.generateIconTemplate({icon:b,title:e.software.title,svgFilter:e.software.svgFilter,link:`/details/software`,count:this.mediaCount?.software})}
        ${this.generateIconTemplate({icon:T,title:e.image.title,svgFilter:e.image.svgFilter,link:`/details/image`,count:this.mediaCount?.image})}
        ${this.generateIconTemplate({icon:D,title:e.etree.title,svgFilter:e.etree.svgFilter,link:`/details/etree`,count:this.mediaCount?.etree})}
        ${this.generateIconTemplate({icon:y,title:e.collection.title,svgFilter:e.collection.svgFilter,link:`/search.php?query=mediatype:collection&sort=-downloads`,count:this.mediaCount?.collection})}
      </div>
    `}generateIconTemplate(e){return a`
      <a href=${e.link} class="media-type" .title=${e.title}>
        <div class="media-icon">
          <img
            src=${e.icon}
            style="filter: ${e.svgFilter};"
            alt=${o(r`${e.title} Image`)}
          />
        </div>
        <div class="media-count">
          ${this.isLoading?a`-`:a`${m(e.count??0,`short`)}`}
        </div>
      </a>
    `}static get styles(){return e`
      :host {
        --heroBlockIconFontColor--: var(
          --heroBlockIconFontColor,
          var(--heroBlockFontColor, var(--ia-theme-primary-text-color, #2c2c2c))
        );
        display: block;
      }

      .media-type {
        display: inline-block;
        text-align: center;
        text-decoration: none;
        width: calc((100% / 9) - (0.8rem));
        margin-right: 0.5rem;
      }

      .media-type:last-child {
        margin-right: 0;
      }

      .media-type:hover {
        cursor: pointer;
      }

      .media-type:hover .media-icon img {
        /* the !important is needed to override the style configuration */
        filter: invert(60%) sepia(13%) saturate(2350%) hue-rotate(182deg)
          brightness(90%) contrast(91%) !important;
      }

      .media-icon {
        width: 100%;
        height: 100%;
        text-align: center;
      }

      .media-count {
        color: var(--heroBlockIconFontColor--);
        font-weight: 200;
      }
    `}};g([t({type:Object})],A.prototype,`mediaCount`,void 0),g([t({type:Boolean})],A.prototype,`isLoading`,void 0),A=g([u(`home-page-hero-block-icon-bar`),i()],A);function j(e,t,n){return Array.from({length:(t-e)/n+1},(t,r)=>e+r*n)}var M=class extends c{constructor(...e){super(...e),this.isLoading=!1}render(){return this.hasContentToShow?a`
      <div id="announcements-container">
        <h1>${o(`Archive News`)}</h1>
        <ul>
          ${this.isLoading?this.loadingTemplate:this.announcementsTemplate}
        </ul>

        <a href="https://blog.archive.org" id="more-announcements"
          >${o(`More posts`)}</a
        >
      </div>
    `:n}get hasContentToShow(){let e=(this.announcements?.length??0)>0;return this.isLoading||e}get announcementsTemplate(){return a`
      ${this.announcements?.map(e=>a`
          <li>
            <a href=${e.link}>${e.title}</a>
          </li>
        `)}
    `}getRandomIntFromInterval(e,t){return Math.floor(Math.random()*(t-e+1)+e)}get loadingTemplate(){return a`
      ${j(1,3,1).map(()=>a` <li>${this.loadingBarsTemplate}</li> `)}
    `}get loadingBarsTemplate(){let e=this.getRandomIntFromInterval(3,4);return a`${j(0,e,1).map((t,n)=>a`
        <div class="loading-bar" style="width: ${n===e?this.getRandomIntFromInterval(35,65):this.getRandomIntFromInterval(75,95)}%"></div>
      `)}`}static get styles(){return e`
      :host {
        --heroBlockAnnouncementsFontColor--: var(
          --heroBlockAnnouncementsFontColor,
          var(--heroBlockFontColor, var(--ia-theme-primary-text-color, #2c2c2c))
        );
        --ia-theme-link-color--: var(--ia-theme-link-color, #4b64ff);
      }

      #announcements-container h1 {
        margin: 0 0 1rem 0;
        font-size: 1.8rem;
        font-weight: 500;
      }

      #announcements-container ul {
        list-style: none;
        margin: 0;
        padding: 0;
      }

      #announcements-container li {
        margin: 0 0 1rem 0;
        padding: 0;
      }

      #announcements-container ul a {
        text-decoration: none;
        color: var(--heroBlockAnnouncementsFontColor--);
        font-size: 1.2rem;
      }

      #announcements-container ul a:hover {
        text-decoration: underline;
      }

      #announcements-container #more-announcements {
        display: block;
        text-align: right;
        font-size: 1.4rem;
        color: var(--ia-theme-link-color--);
        text-decoration: none;
      }

      #announcements-container #more-announcements:hover {
        text-decoration: underline;
      }

      #announcements-container .loading-bar {
        background-color: lightgray;
        height: 0.75rem;
      }

      #announcements-container .loading-bar:not(:last-child) {
        margin-bottom: 0.3rem;
      }
    `}};g([t({type:Array})],M.prototype,`announcements`,void 0),g([t({type:Boolean})],M.prototype,`isLoading`,void 0),M=g([u(`hero-block-announcements`),i()],M);var N=function(e){return e.Desktop=`desktop`,e.Mobile=`mobile`,e}({}),P=class extends c{constructor(...e){super(...e),this.mediaCountsLoading=!1,this.announcementsLoading=!1}render(){return a`
      <div id="outer-container" class="${l(this.orientation)}">
        <div
          id="section-container"
          class=${this.hasAnnouncementsToShow?``:`no-announcements`}
        >
          <div id="branding">
            ${this.heroLinkUrl?this.clickableHeroTemplate:this.staticHeroTemplate}
          </div>
          <div id="info-search">
            <h1 id="whoweare">
              ${o(a`<strong>Internet Archive</strong> is a non-profit library
                  of millions of free texts, movies, software, music, websites,
                  and more.`)}
            </h1>
            <div id="icon-block-container">
              <home-page-hero-block-icon-bar
                ?isLoading=${this.mediaCountsLoading}
                .mediaCount=${this.mediaCounts}
              ></home-page-hero-block-icon-bar>
            </div>
            <div id="search-container">
              <ia-dropdown-search-bar
                .categories=${this.defaultCategories}
                @searchRequested=${this.searchRequested}
                @categoryChanged=${this.categoryChanged}
              ></ia-dropdown-search-bar>
              ${this.advancedSearchTemplate}
            </div>
          </div>
          ${this.hasAnnouncementsToShow?this.announcementsTemplate:n}
        </div>
      </div>
    `}get hasAnnouncementsToShow(){let e=(this.announcements?.length??0)>0;return this.announcementsLoading||e}get announcementsTemplate(){return a`
      <div
        id="announcements"
        class=${this.announcementsLoading?`loading`:``}
      >
        <hero-block-announcements
          .announcements=${this.announcements}
          ?isLoading=${this.announcementsLoading}
        ></hero-block-announcements>
      </div>
    `}get advancedSearchTemplate(){let{navBaseUrl:e}=this;return e===void 0?n:a`<a
      id="advanced-search-link"
      href=${`${e}/advancedsearch.php`}
      @click=${this.advancedSearchClicked}
    >
      ${o(`Advanced Search`)}
    </a>`}get defaultCategories(){return k(this.orientation===`mobile`)}get externalSearchHandlers(){return{web:this.waybackSearchHandler}}searchRequested(e){let{query:t,category:n}=e.detail,r=this.externalSearchHandlers[n];if(r&&t)r.performSearch(t);else if(t){let e=new URLSearchParams({query:t,tab:n});window.location.href=`${this.navBaseUrl??``}/search?${e}`}this.dispatchEvent(new Event(`searchPerformed`))}categoryChanged(e){this.dispatchEvent(new CustomEvent(`categoryChanged`,{detail:e.detail}))}advancedSearchClicked(){this.dispatchEvent(new CustomEvent(`advancedSearchClicked`))}heroLinkClicked(e){e.preventDefault(),this.dispatchEvent(new CustomEvent(`heroLinkClicked`,{detail:{url:this.heroLinkUrl}}))}get logoSrc(){return this.heroLogoSrc||`data:image/svg+xml,%3csvg%20height='86'%20viewBox='0%200%2076%2086'%20width='76'%20xmlns='http://www.w3.org/2000/svg'%3e%3cpath%20d='m76%2082v4h-76l.00080851-4zm-3-6v5h-70v-5zm-62.6696277-54%20.8344146.4217275.4176066%206.7436084.4176065%2010.9576581v10.5383496l-.4176065%2013.1364492-.0694681%208.8498268-1.1825531.3523804h-4.17367003l-1.25202116-.3523804-.48627608-8.8498268-.41840503-13.0662957v-10.5375432l.41840503-11.028618.38167482-6.7798947.87034634-.3854412zm60.0004653%200%20.8353798.4217275.4168913%206.7436084.4168913%2010.9576581v10.5383496l-.4168913%2013.1364492-.0686832%208.8498268-1.1835879.3523804h-4.1737047l-1.2522712-.3523804-.4879704-8.8498268-.4168913-13.0662957v-10.5375432l.4168913-11.028618.3833483-6.7798947.8697215-.3854412zm-42.000632%200%20.8344979.4217275.4176483%206.7436084.4176482%2010.9576581v10.5383496l-.4176482%2013.1364492-.0686764%208.8498268-1.1834698.3523804h-4.1740866l-1.2529447-.3523804-.4863246-8.8498268-.4168497-13.0662957v-10.5375432l.4168497-11.028618.38331-6.7798947.8688361-.3854412zm23%200%20.8344979.4217275.4176483%206.7436084.4176482%2010.9576581v10.5383496l-.4176482%2013.1364492-.0686764%208.8498268-1.1834698.3523804h-4.1740866l-1.2521462-.3523804-.4871231-8.8498268-.4168497-13.0662957v-10.5375432l.4168497-11.028618.38331-6.7798947.8696347-.3854412zm21.6697944-9v7h-70v-7zm-35.7200748-13%2036.7200748%208.4088317-1.4720205%202.5911683h-70.32799254l-2.19998696-2.10140371z'%20fill='%232c2c2c'%20fill-rule='evenodd'/%3e%3c/svg%3e`}get clickableHeroTemplate(){let e=this.heroLinkAriaLabel||o(`Internet Archive Logo`);return a`
      <a
        href=${this.heroLinkUrl}
        @click=${this.heroLinkClicked}
        aria-label=${e}
        class="hero-link"
      >
        ${this.heroFigureTemplate(this.logoSrc)}
      </a>
    `}get staticHeroTemplate(){return this.heroFigureTemplate(this.logoSrc)}heroFigureTemplate(e){return a`
      <figure>
        <img
          src=${e}
          id="main-image"
          alt=${o(`Internet Archive Logo`)}
        />
        ${this.heroCaption?a`<figcaption class="branding-caption">
              ${this.heroCaption}
            </figcaption>`:n}
      </figure>
    `}static get styles(){return e`
      :host {
        --heroBlockBackgroundColor--: var(
          --heroBlockBackgroundColor,
          var(--ia-theme-secondary-background-color, white)
        );
        --heroBlockFontColor--: var(
          --heroBlockFontColor,
          var(--ia-theme-primary-text-color, #2c2c2c)
        );
        --heroBlockSeparatorStyle--: var(
          --heroBlockSeparatorStyle,
          1px solid #aaa
        );
        --ia-theme-link-color--: var(--ia-theme-link-color, #4b64ff);

        display: block;
      }

      a:link {
        text-decoration: none;
        display: block;
        color: var(--ia-theme-link-color--);
        white-space: nowrap;
      }

      a:hover {
        text-decoration: underline;
      }

      a:visited {
        color: var(--ia-theme-link-color--, #4b64ff);
      }

      #outer-container {
        padding: 3rem 1rem;
        background-color: var(--heroBlockBackgroundColor--);
        box-shadow: rgb(204, 204, 204) 1px 1px 2px 0px;
      }

      #section-container {
        display: flex;
        flex-direction: row;
      }

      .mobile #section-container {
        flex-direction: column;
      }

      #branding {
        margin: 0 1.5rem;
        flex: 1 0 20%;
        color: var(--ia-theme-link-color--);
        display: flex;
        justify-content: center;
      }
      #branding .hero-link {
        text-decoration: none;
        display: inline-block;
        color: inherit;
      }
      #branding .hero-link:hover figure {
        text-decoration: underline;
        cursor: pointer;
      }
      #branding .hero-link:focus {
        outline: 2px solid var(--ia-theme-link-color--);
        outline-offset: 4px;
      }
      #branding img {
        width: 145px;
        height: auto;
        margin-bottom: 2rem;
        margin-top: 2rem;
      }
      #branding .branding-caption {
        width: max-content;
        font-size: 1.6rem;
        font-weight: bold;
        white-space: inherit;
        margin: 0.5rem auto;
        color: var(--heroBlockFontColor--);
      }
      .branding-caption .caption-break {
        display: inline;
      }
      .mobile .branding-caption .caption-break {
        display: none;
      }

      .mobile #branding {
        margin-bottom: 2rem;
      }
      .mobile #branding img {
        margin-top: 0;
        min-height: 19rem;
      }
      .mobile .branding-caption {
        padding: 0.2em 0.2em;
      }

      #info-search {
        padding: 0 4rem;
        border-left: var(--heroBlockSeparatorStyle--);
        border-right: var(--heroBlockSeparatorStyle--);
      }

      .mobile #info-search {
        border: 0;
        padding: 0 1rem;
        margin-bottom: 2rem;
      }

      .no-announcements #info-search {
        /* Without the announcements column, this border would sit at the
           outer edge of the hero block with nothing to its right. */
        border-right: 0;
      }

      #whoweare {
        font-size: 2.6rem;
        line-height: 1.1;
        font-weight: 300;
        margin: 0 0 2rem 0;
        color: var(--heroBlockFontColor--);
      }

      #icon-block-container {
        text-align: center;
        margin-bottom: 2rem;
      }

      home-page-hero-block-icon-bar {
        min-width: 26rem;
        max-width: 44rem;
        margin: auto;
      }

      ia-dropdown-search-bar {
        --search-bar-width: 100%;
        --search-bar-height: 5rem;
        --padding-sm: 10px;
        --clear-button-width: 3rem;
        --clear-button-height: 3rem;
        --clear-button-offset: 9px;
        --dropdownFontSize: 1.4rem;
        --dropdownItemButtonPadding: 0 10px;
        font-size: 1.7rem;
      }

      #advanced-search-link {
        float: right;
        margin: 10px 0 2px;
        font-size: 1.4rem;
      }

      #announcements {
        margin: 0 4rem;
        flex: 1 0 20%;
      }

      #announcements.loading {
        min-width: 17rem;
      }

      .mobile #announcements {
        margin: 0 1rem;
      }
    `}};g([t({type:String})],P.prototype,`orientation`,void 0),g([t({type:Object})],P.prototype,`mediaCounts`,void 0),g([t({type:Array})],P.prototype,`announcements`,void 0),g([t({type:Object})],P.prototype,`waybackSearchHandler`,void 0),g([t({type:Object})],P.prototype,`abtestManager`,void 0),g([t({type:Boolean})],P.prototype,`mediaCountsLoading`,void 0),g([t({type:Boolean})],P.prototype,`announcementsLoading`,void 0),g([t({type:String})],P.prototype,`navBaseUrl`,void 0),g([t({type:String})],P.prototype,`heroLogoSrc`,void 0),g([t({type:String})],P.prototype,`heroCaption`,void 0),g([t({type:String})],P.prototype,`heroLinkUrl`,void 0),g([t({type:String})],P.prototype,`heroLinkAriaLabel`,void 0),P=g([u(`home-page-hero-block`),i()],P);var F=class{cellForIndex(e){let t=this.items[e];return a`
      <tile-dispatcher
        tileDisplayMode="grid"
        .model=${{identifier:t.identifier,mediatype:`collection`,title:t.title?.value,itemCount:t.item_count?.value}}
        .baseNavigationUrl=${this.navUrl}
        .baseImageUrl=${this.imageUrl}
      >
      </tile-dispatcher>
    `}constructor(e){this.items=e.items,this.navUrl=e.navigationUrl,this.imageUrl=e.imageUrl,this.itemCount=e.items.length}},I=class{constructor(e){this.baseUrl=e}getJsonLdContent(){return{"@type":`WebSite`,name:`Internet Archive`,alternateName:`Archive.org`,url:`https://${this.baseUrl}`,potentialAction:{"@type":`SearchAction`,target:`https://${this.baseUrl}/search?query={query}`,"query-input":`required name=query`}}}},L=[{id:`search`,body:()=>o(a`How to search the archive`),icon:`data:image/svg+xml,%3csvg%20viewBox='0%200%20100%20100'%20xmlns='http://www.w3.org/2000/svg'%3e%3cpath%20d='m17.0555551%2041.3194459c0-12.7430552%2010.3541664-23.1027772%2023.0847216-23.1027772%2012.7166664%200%2023.0777773%2010.359722%2023.0777773%2023.1027772%200%2012.7361108-10.3611109%2023.0986106-23.0777773%2023.0986106-12.7305552%200-23.0847216-10.3624998-23.0847216-23.0986106zm-17.24305512%200c0%2022.2916661%2018.04583292%2040.3472213%2040.32777672%2040.3472213%208.9208332%200%2017.145833-2.9319449%2023.8194439-7.8527776l24.1513883%2024.0777771c1.2125%201.1402778%202.8430555%201.8430556%204.6374999%201.8430556%203.7444443%200%206.7805554-3.0361111%206.7805554-6.7791665%200-2.0652778-.9222222-3.9069444-2.3736111-5.1499999l-23.718055-23.7458328c4.4152777-6.4791665%207.0152776-14.3055552%207.0152776-22.7402772%200-22.2791661-18.0458328-40.34861006-40.312499-40.34861006-22.2819438%200-40.32777672%2018.06944396-40.32777672%2040.34861006z'%20fill='%23000'%20fill-rule='evenodd'/%3e%3c/svg%3e`,link:`https://help.archive.org/help/search-a-basic-guide/`},{id:`download`,body:()=>o(a`How to download files`),icon:`data:image/svg+xml,%3csvg%20viewBox='0%200%20100%20100'%20xmlns='http://www.w3.org/2000/svg'%3e%3cpath%20d='m50%2080%2033.3333333-43.3333333h-20v-36.6666667h-26.6666666v36.6666667h-20zm50%2020v-13.3333333h-100v13.3333333z'%20fill='%23000'%20fill-rule='evenodd'/%3e%3c/svg%3e`,link:`https://help.archive.org/help/how-to-download-files/`},{id:`listen-music`,body:()=>o(a`Listening to music on the archive`),icon:`data:image/svg+xml,%3csvg%20viewBox='0%200%20100%20100'%20xmlns='http://www.w3.org/2000/svg'%3e%3cpath%20d='m31.9543923%2073.9149367c0-18.440045.603017-35.3592066.3602093-52.5745402-.0773783-5.653951%204.1530791-8.1634087%2010.2352789-9.3814496%2017.2673638-3.42065373%2037.0001598-7.82587936%2054.6584174-11.9589469%200%2023.9886015.5483185%2047.2060884-.2161255%2070.3795497-.4095713%2012.145722-20.1890611%2020.8627857-31.2274732%2015.3022223-5.333765-2.6908965-8.0860305-7.2201942-5.8207146-13.6265823%203.5820808-10.187251%2014.2989729-15.5196819%2025.5108191-12.5673002%201.4061502.3735503%202.796291.8204766%205.3524425%201.5755819%200-14.0534969.0173434-27.5013087-.0200116-40.9531228%200-1.4408369-.3028426-2.8870103-.5816712-5.3017464-17.1019344%203.8048769-33.7516061%207.5043593-51.4939125%2011.4479834%200%2015.9732788.3548728%2031.6904088-.1947798%2047.3808566-.197448%205.7660161.985906%2014.078845-3.4766863%2018.8242682-5.7326634%206.0928726-16.7590685%208.9091756-24.6890083%206.8920035-6.59583148-1.6836447-10.0845247-9.2440364-7.48968404-16.2494392%203.30325217-8.9465303%2015.93192144-13.9307589%2025.10391604-11.2131803%201.8930997.5563232%203.9889838%202.0238423%203.9889838%202.0238423'%20fill-rule='evenodd'/%3e%3c/svg%3e`,link:`https://help.archive.org/help/media-players-a-basic-guide/`},{id:`find-web-pages`,body:()=>o(a`How do I find old web pages?`),icon:`data:image/svg+xml,%3csvg%20viewBox='0%200%20100%20100'%20xmlns='http://www.w3.org/2000/svg'%3e%3cpath%20d='m93%2021h-11v11h11zm-17%200h-11v11h11zm17%2058v-37h-86v37zm-32.9981573-58h-53.00000002v11h53.00000002zm39.9981573%2065h-100v-72h100z'/%3e%3c/svg%3e`,link:`https://help.archive.org/help/using-the-wayback-machine/`},{id:`upload`,body:()=>o(a`How to upload to the archive`),icon:`data:image/svg+xml,%3csvg%20viewBox='0%200%20100%20100'%20xmlns='http://www.w3.org/2000/svg'%3e%3cpath%20d='m50%2020%2033.3333333%2043.3333333h-20v36.6666667h-26.6666666v-36.6666667h-20zm50-20v13.3333333h-100v-13.3333333z'%20fill='%23000'%20fill-rule='evenodd'/%3e%3c/svg%3e`,link:`https://help.archive.org/help/uploading-a-basic-guide/`},{id:`print-disability`,body:()=>o(a`Print disability resources`),icon:`data:image/svg+xml,%3csvg%20viewBox='0%200%20100%20100'%20xmlns='http://www.w3.org/2000/svg'%3e%3cpath%20d='m75.2264339%2043.7758621v34.5172413c-.0876744%201.131666-.3170056%201.8787924-.6879937%202.2413794-.3709881.3625869-1.1354255.5867248-2.2933121.6724138h-14.9065292c-1.3012145.3659876-2.2185394.6648382-2.7519746.8965517-.5334351.2317135-.9920976.530564-1.3759873.8965517h-2.0639809v-36.7586207c1.0988946-1.0464298%201.9397757-1.7188436%202.5226434-2.0172414.5828676-.2983977%201.347305-.447823%202.2933121-.4482758zm-31.8770392%200c.9460072.0004528%201.7104445.1498781%202.2933122.4482758.5412342.2770836%201.3049393.8766607%202.2911151%201.7987312l.2315283.2185102v36.7586207h-2.063981c-.3838897-.3659877-.8425521-.6648382-1.3759873-.8965517-.4924017-.2138894-1.3119047-.4849843-2.4585089-.8132848l-.2934657-.0832669h-14.9065291c-1.1578867-.085689-1.9223241-.3098269-2.2933122-.6724138-.3424505-.3346957-.5641994-.9970463-.6652465-1.9870518l-.0227471-.2543276v-34.5172413zm6.6506053-25.7758621c21.7118911%200%2039.8283179%2014.4313115%2044.0301301%2033.6201171l1.1481197.0005726.1834922.0443783c1.0423285.2906436%202.1278205%201.2471151%203.256476%202.8694148%201.1907821%201.7115987%201.3712037%205.6822773%201.381782%207.53079l-.0000265.362031c-.0010068.1631778-.0032741.2984015-.0057682.4002824-.11035%204.5077624-1.9615514%207.4562027-2.7519746%208.2931035-1.7713859%201.8755449-4.6117117%202.8456544-8.5209772%202.9103284l-.4229402.0034647h-3.8986307v-23.0862069l3.5804271-.0002345c-1.7734436-6.4584284-5.37237-12.2299166-10.2504661-16.8023803-7.0966346-6.652002-16.9005442-10.7663508-27.7296436-10.7663508s-20.633009%204.1143488-27.7296436%2010.7663508c-4.8780961%204.5724637-8.4770225%2010.3439519-10.2504661%2016.8023803l3.5804271.0002345v23.0862069h-3.8986307l-.4229402-.0034647c-3.90926551-.064674-6.74959126-1.0347835-8.52097722-2.9103284-.79042321-.8369008-2.64162462-3.7853411-2.7519746-8.2931035-.00249404-.1018809-.00476136-.2371046-.00576814-.4002824l-.00002654-.362031c.01057834-1.8485127.19099987-5.8191913%201.38178198-7.53079%201.12865552-1.6222997%202.21414752-2.5787712%203.256476-2.8694148l.18349226-.0443783%201.14811963-.0005726c4.20181223-19.1888056%2022.31823903-33.6201171%2044.03013013-33.6201171z'%20fill-rule='evenodd'/%3e%3c/svg%3e`,link:`https://help.archive.org/help/print-disability-access-general-information/`},{id:`account-settings`,body:()=>o(a`How to change your account settings`),icon:`data:image/svg+xml,%3csvg%20viewBox='0%200%20100%20100'%20xmlns='http://www.w3.org/2000/svg'%3e%3cpath%20d='m89.6854559%2079.6500588c1.7300364%206.4823648%202.180423%2013.3122689%203.3145441%2020.3499412h-86c.5683151-15.8558542%202.98334063-30.7849367%2015.1676149-41.6581341%2022.9948067-20.518674%2059.250299-9.0032844%2067.517841%2021.3081929zm-40.0998307-79.6500588c10.872402.0493248%2019.9700408%209.25722341%2019.917959%2020.1421788-.0829413%2011.042868-8.9616237%2019.8492523-20.0602807%2019.8578212-11.1181198%200-19.9397193-8.7904706-19.9397193-19.8908727-.0327543-11.11998815%209.0125781-20.17487063%2020.082041-20.1091273z'%20fill='%23000'%20fill-rule='evenodd'/%3e%3c/svg%3e`,link:`https://help.archive.org/help/accounts-tips-troubleshooting/`},{id:`other-questions`,body:()=>o(a`Other common questions`),icon:`data:image/svg+xml,%3csvg%20viewBox='0%200%20100%20100'%20xmlns='http://www.w3.org/2000/svg'%3e%3cpath%20d='m50.2918908%202h.1658384c8.7295471%200%2016.8199256%202.15471102%2024.2711354%206.46413307%207.4512098%204.30942203%2013.3453816%2010.17585603%2017.6825154%2017.59930193%204.3371339%207.4234458%206.5333405%2015.4998694%206.58862%2024.2292705%200%206.4491618-1.2092381%2012.634599-3.6277142%2018.5563115-2.4184761%205.9217124-5.9079917%2011.1904473-10.4685467%2015.8062045-9.3975072%209.4526286-20.7954397%2014.2342214-34.1937974%2014.3447785h-.2487576c-8.7295471%200-16.8061057-2.154711-24.2296758-6.4641331-7.4235701-4.309422-13.3177419-10.1896756-17.68251546-17.6407608-4.36477358-7.4510851-6.54716037-15.5413283-6.54716037-24.2707294-.05527946-6.4491618%201.14013874-12.634599%203.5862546-18.5563114%202.44611585-5.9217125%205.94945123-11.1616565%2010.51000623-15.7198319%204.5582517-4.6157573%209.7844634-8.16049298%2015.6786353-10.63420719%205.8941718-2.4737142%2012.0658925-3.7105713%2018.5151622-3.7105713zm6.3364075%2064.7207936v-3.6691124c.33398-2.056822.9731487-3.6691125%201.917506-4.8368714.8337985-1.1677589%201.9739372-2.3078786%203.4204163-3.420359%201.7781557-1.3335946%203.140564-2.3343663%204.0872246-3.0023152%201.4464791-.9443415%202.7812476-2.167379%204.0043055-3.6691124%201.1677785-1.4464548%202.1685669-3.1428147%203.0023654-5.0890796.7785189-1.8909864%201.1677784-4.44877%201.1677784-7.6733509%200-2.0015435-.3892595-4.0030869-1.1677784-6.0046303-.8337985-2.1121005-2.168567-4.1412832-4.0043055-6.0875481-2.0015769-1.9462649-4.4753325-3.5032768-7.4212668-4.6710358-3.1693553-1.278316-6.9778794-1.917474-11.4255722-1.917474-3.3927765%200-6.6174113.6115188-9.6739045%201.8345562-2.7800959%201.167759-5.2262117%202.835328-7.3383476%205.002707-2.0568563%202.1121005-3.6138943%204.6146056-4.6711138%207.5075153-1.1124991%202.8353279-1.724028%206.0322696-1.8345869%209.5908249h14.17918c0-1.5570119.1946297-3.1969416.5838892-4.9197891.444539-1.6122905%201.0560679-2.9193974%201.8345869-3.9213207.7232396-1.1124804%201.7516677-2.0303344%203.0852846-2.7535618%201.2230579-.6679489%202.7248164-1.0019233%204.5052755-1.0019233%202.5566748%200%204.5582517.7232274%206.0047307%202.1696822%201.4464791%201.3911764%202.1697186%203.559707%202.1697186%206.505592.1105589%201.778126-.1957814%203.2234293-.9190209%204.3359097-.7232395%201.278316-1.6123174%202.3620055-2.6672337%203.2510685-1.6675969%201.4464549-2.8353753%202.4195873-3.5033354%202.9193974-1.3336168.9443415-2.5566748%202.0844612-3.6691738%203.420359-1.2783374%201.4464548-2.3067656%203.031106-3.0852845%204.7539535-.8890779%201.9462649-1.4176877%204.2541436-1.5858294%206.9236359v4.4188275h13.0114016zm.8361017%205.752422h-15.1776652v14.9286579h15.1776652z'/%3e%3c/svg%3e`,link:`https://help.archive.org/help/category/archive-org/most-frequently-asked-questions-archive-org/`}],R=class extends c{render(){return a`
      <div id="onboarding-tile-icon">
        <img src=${this.content.icon} alt="" />
      </div>
      <div id="onboarding-tile-body">${this.content.body()}</div>
    `}static get styles(){return e`
      :host {
        display: flex;
        align-items: stretch;
        height: 100%;
        border: 2px solid #6d84a0;
        border-radius: 5px;
        background-color: white;
        cursor: pointer;
      }
      #onboarding-tile-icon {
        flex: 0 0 var(--tile-height);
        padding: 5px;
        background-color: #d9edf5;
        border-radius: 3px 0px 0px 3px;
      }
      #onboarding-tile-icon > img {
        width: 100%;
        height: 100%;
        filter: invert(20%) sepia(32%) saturate(2756%) hue-rotate(190deg)
          brightness(98%) contrast(90%); /* Matches #194880 */
      }
      #onboarding-tile-body {
        flex: 1 1 auto;
        display: flex;
        align-items: center;
        padding: 5px;
        font-size: 1.4rem;
        color: #2c2c2c;
        overflow: wrap;
      }
    `}};g([t({attribute:!1})],R.prototype,`content`,void 0),R=g([u(`onboarding-tile`),i()],R);var z=class extends c{render(){return a`
      <div id="home-page-onboarding" class="carousel-container">
        <basic-carousel
          .resizeObserver=${this.resizeObserver}
          .carouselTitle=${o(`New to the Archive?`)}
        >
          ${L.map(e=>a`
              <a class="onboarding-link" href="${e.link}">
                <onboarding-tile
                  .content=${e}
                  @click=${()=>this.onTileClick(e)}
                ></onboarding-tile>
              </a>
            `)}
        </basic-carousel>
      </div>
    `}async onTileClick(e){(await(await this.appServices?.get())?.analyticsHandler.get())?.sendEvent({category:`HomePage-Onboarding`,action:`Onboarding-${e.id}`,label:location.href})}static get styles(){return e`
      :host {
        display: block;
        --num-tiles: 1.75;
        --tile-width: 240px;
        --tile-height: 60px;
        --tile-gap: 10px;
      }
      /* Breakpoint sizes come from Bootstrap */
      @media screen and (min-width: 576px) and (max-width: 768px) {
        :host {
          --num-tiles: 1.75;
        }
      }
      @media screen and (min-width: 768px) and (max-width: 992px) {
        :host {
          --num-tiles: 2;
        }
      }
      @media screen and (min-width: 992px) and (max-width: 1200px) {
        :host {
          --num-tiles: 3;
        }
      }
      @media screen and (min-width: 1200px) {
        :host {
          --num-tiles: 4;
        }
      }

      .carousel-container {
        display: flex;
        justify-content: center;
      }

      basic-carousel {
        --basic-carousel-tile-gap: var(--tile-gap);
        --basic-carousel-content-width: calc(
          var(--num-tiles) * var(--tile-width) + (var(--num-tiles) - 1) *
            var(--tile-gap)
        );
        --basic-carousel-title-text-align: center;
      }

      .onboarding-link {
        text-decoration: none;
      }

      onboarding-tile {
        box-sizing: border-box;
        width: var(--tile-width);
        height: var(--tile-height);
        flex-shrink: 0;
        scroll-snap-align: start; /* Snaps to L edge of panel */
      }
    `}};g([t({type:Object})],z.prototype,`appServices`,void 0),g([t({type:Object})],z.prototype,`resizeObserver`,void 0),z=g([u(`home-page-onboarding`),i()],z);var B=class extends c{constructor(...e){super(...e),this.resizeObserverInitialized=!1,this.collectionsLoading=!1,this.mediaCountLoading=!1,this.announcementsLoading=!0,this.currentPage=1,this.lastPageReached=!1,this.isPrerendering=!1,this.shouldShowOnboardingCarousel=!0,this.tileCountToFetch=50,this.handleNoNewsPostsOverrideChanged=()=>{this.fetchAnnouncements()},this.fetchAnnouncementsRequestId=0}render(){return a`
      <div id="wayback-search-container">${this.waybackTemplate}</div>
      <div id="page-container">
        <div id="page-content-container">
          ${this.mainTemplateDependenciesReady?this.mainTemplate:n}
        </div>
      </div>
    `}get waybackTemplate(){return a`
      <ia-wayback-search-form
        waybackPagesArchived=${this.waybackPageCount===void 0?``:m(this.waybackPageCount)}
        .queryHandler=${{performQuery:e=>{this.waybackSearchHandler?.performSearch(e)}}}
      ></ia-wayback-search-form>
    `}get mainTemplate(){return a`
      <home-page-hero-block
        .orientation=${this.heroBlockOrientation}
        .announcements=${this.announcements}
        .mediaCounts=${this.mediaCounts}
        .navBaseUrl=${this.navBaseUrl}
        .abtestManager=${this.abtestManager}
        .waybackSearchHandler=${this.waybackSearchHandler}
        .tvSearchHandler=${this.tvSearchHandler}
        .heroLogoSrc=${this.heroLogoSrc}
        .heroCaption=${this.heroCaption}
        .heroLinkUrl=${this.heroLinkUrl}
        .heroLinkAriaLabel=${this.heroLinkAriaLabel}
        ?announcementsLoading=${this.announcementsLoading}
        ?mediaCountsLoading=${this.mediaCountLoading}
        @searchPerformed=${this.heroBlockSearchPerformed}
        @categoryChanged=${this.heroBlockCategoryChanged}
        @advancedSearchClicked=${this.heroBlockAdvancedSearchClicked}
        @heroLinkClicked=${this.heroLinkClicked}
      ></home-page-hero-block>

      ${this.shouldShowOnboardingCarousel===!0?this.onboardingCarouselTemplate:n}

      <h1>${o(`Top Collections`)}</h1>

      ${this.homePageServices?a`<infinite-scroller
            .cellProvider=${this.infiniteScrollerDataSource}
            ?scrollOptimizationsDisabled=${this.isPrerendering}
            @scrollThresholdReached=${this.performSearch}
            @cellSelected=${this.cellSelected}
          ></infinite-scroller>`:n}
      ${this.infiniteScrollActivityTemplate}
    `}get onboardingCarouselTemplate(){return a`
      <home-page-onboarding
        .appServices=${this.appServices}
        .resizeObserver=${this.resizeObserver}
      ></home-page-onboarding>
    `}get infiniteScrollActivityTemplate(){let e=a`${n}`;return this.lastPageReached?e=a`No more items`:this.collectionsLoading&&(e=a`<ia-status-indicator
        class="loading-indicator"
        mode="loading"
        ?hideDots=${!0}
      ></ia-status-indicator>`),a`<div id="activity-indicator-container">${e}</div>`}connectedCallback(){super.connectedCallback(),window.addEventListener(v,this.handleNoNewsPostsOverrideChanged)}disconnectedCallback(){window.removeEventListener(v,this.handleNoNewsPostsOverrideChanged),super.disconnectedCallback()}firstUpdated(){let e=new I(this.apiBaseUrl??`archive.org`);new p().applyMetadata(e)}updated(e){e.has(`appServices`)&&this.setupAppServicesDependents()}handleResize(e){let t=e.contentRect.width+(this.appRootController?.getWindowScrollbarWidth()??0);switch(e.target){case this.pageContainer:this.heroBlockOrientation=t<=800?N.Mobile:N.Desktop,this.resizeObserverInitialized||(this.resizeObserverInitialized=!0);break;default:break}}async cellSelected(){this.analyticsHandler?.sendEvent({category:`GenericCollection`,action:`ItemTile`})}heroLinkClicked(e){let t=e.detail.url;this.analyticsHandler?.sendEvent({category:`HomePage`,action:`HeroClick`}),this.navigateToUrl(t)}navigateToUrl(e){window.location.href=e}async heroBlockSearchPerformed(){this.analyticsHandler?.sendEvent({category:`Home`,action:`SearchForm`})}heroBlockCategoryChanged(e){this.analyticsHandler?.sendEvent({category:`Home`,action:`SearchCategoryChanged`,label:e.detail})}heroBlockAdvancedSearchClicked(){this.analyticsHandler?.sendEvent({category:`Home`,action:`AdvancedSearchClicked`})}disconnectResizeObserver(){this.resizeObserver?.removeObserver({handler:this,target:this.pageContainer})}async setupResizeObserver(){this.disconnectResizeObserver();let e=await this.appServices?.get();this.appRootController=await e?.appRootController.get(),this.resizeObserver=await e?.resizeObserver.get(),this.resizeObserver?.addObserver({handler:this,target:this.pageContainer})}async setupAppServicesDependents(){let e=await this.appServices?.get();await Promise.all([this.setupIsPrerendering(),this.setupResizeObserver(),this.setupNavBaseUrl(),this.setupTopNavAndFooter(),this.setupPageMetadata(),this.setupHomePageServices(),this.abtestManager=await e?.abtestManager.get(),this.analyticsHandler=await e?.analyticsHandler.get(),this.waybackSearchHandler=await e?.waybackSearchHandler.get(),this.tvSearchHandler=await e?.tvSearchHandler.get()])}async setupIsPrerendering(){let e=await(await this.appServices?.get())?.appConfig.get();this.isPrerendering=e?.isPrerendering??!1}async setupNavBaseUrl(){let e=await(await this.appServices?.get())?.appConfig.get();this.navBaseUrl=e?.archiveNavUrl,this.apiBaseUrl=e?.archiveApiUrl}async setupHomePageServices(){let e=await this.appServices?.get(),t=await e?.fetchHandler.get();if(!t)return;let[n,r,i]=await Promise.all([f(()=>h(()=>import(`./home-page-services-3ps3baAX.js`),__vite__mapDeps([0,1,2,3,4]))),e?.keyValueStoreManager.get(),e?.appConfig.get()]),a=this.resolveDevMenuEnabled(e,i?.environment);this.homePageServices=new n.HomePageServices(t,{keyValueStoreManager:r,devMenuEnabled:a}),this.fetchMediaCounts(),this.fetchAnnouncements()}async resolveDevMenuEnabled(e,t){return t===`dev`?!0:_(t,(await(await e?.userService.get())?.getLoggedInUser())?.success?.privs)}async setupTopNavAndFooter(){(await(await this.appServices?.get())?.appRootController.get())?.setAppFooterMode(`fixed`)}async setupPageMetadata(){let e=await(await this.appServices?.get())?.pageMetadataService.get();e?.setMetaTag({name:`monetization`,content:`$ilp.uphold.com/D7BwPKMQzBiD`}),e?.setMetaTag({name:`description`,content:`Internet Archive is a non-profit digital library offering free universal access to texts, movies & music, as well as 624 billion archived web pages.`}),e?.addLink({rel:`alternate`,type:`application/xml`,title:`RSS`,path:`/services/collection-rss.php`})}get mainTemplateDependenciesReady(){return this.resizeObserverInitialized&&this.shouldShowOnboardingCarousel!==void 0}async performSearch(){if(!this.homePageServices||this.lastPageReached)return;this.collectionsLoading=!0;let e=await this.homePageServices.fetchCollections(this.currentPage,this.tileCountToFetch);if(!e){this.collectionsLoading=!1;return}let t=(this.infiniteScrollerDataSource?.items??[]).concat(e),n=await(await this.appServices?.get())?.appConfig.get();this.infiniteScrollerDataSource=new F({items:t,navigationUrl:n?.archiveNavUrl,imageUrl:n?.baseImageUrl}),this.infiniteScroller.itemCount=this.infiniteScrollerDataSource.itemCount,this.collectionsLoading=!1,e.length<this.tileCountToFetch&&(this.lastPageReached=!0),this.currentPage+=1}async fetchMediaCounts(){this.mediaCountLoading=!0;let e=await(await this.appServices?.get())?.mediaCountService.get();this.mediaCounts=await e?.fetchMediaCounts(),this.mediaCountLoading=!1,this.waybackPageCount=this.mediaCounts?.web??0}async fetchAnnouncements(){if(!this.homePageServices)return;let e=++this.fetchAnnouncementsRequestId;this.announcementsLoading=!0;try{let t=await this.homePageServices.fetchAnnouncements();if(e!==this.fetchAnnouncementsRequestId)return;this.announcements=t}catch(e){console.warn(`Unable to load Archive News announcements:`,e)}finally{e===this.fetchAnnouncementsRequestId&&(this.announcementsLoading=!1)}}static get styles(){return e`
      :host {
        display: block;
      }

      #wayback-search-container {
        padding: 1.5rem 1.5rem 5rem 1.5rem;
        margin-bottom: 1rem;
        color: white;
        background: #666;
      }

      @media (max-width: 900px) {
        #wayback-search-container {
          padding-bottom: 2.5rem;
        }
      }

      #page-container {
        max-width: min(var(--app-page-max-width, 970px), 90%);
        margin: 2rem auto;
      }

      @media (max-width: 600px) {
        #page-container {
          max-width: 100%;
          width: 100%;
          margin: 1rem auto;
        }
      }

      infinite-scroller {
        max-width: 100rem;
        display: block;
        margin: auto;
      }

      @media (max-width: 400px) {
        infinite-scroller {
          --infiniteScrollerCellMinWidth: 12rem;
        }
      }

      ia-wayback-search-form {
        --themeFontFamily: var(--ia-theme-base-font-family);
        /* The desktop search icon is black on the home page. */
        --desktopSearchIconFill: #000;
        display: block;
        max-width: 60rem;
        margin: auto;
        text-align: center;
      }

      #activity-indicator-container {
        margin: 0rem auto 4rem auto;
        font-size: 1.6rem;
        text-align: center;
      }

      .loading-indicator {
        --icon-width: 4rem;
        --primary-text-color: var(--ia-theme-loading-color, dodgerblue);
        margin: auto;
        display: inline-block;
      }

      #page-container h1,
      #page-container h2 {
        font-weight: 200;
        font-size: 2.4rem;
        margin-top: 2rem;
        margin-bottom: 1rem;
        text-align: center;
      }
    `}};g([t({type:Object})],B.prototype,`appServices`,void 0),g([s()],B.prototype,`resizeObserverInitialized`,void 0),g([s()],B.prototype,`infiniteScrollerDataSource`,void 0),g([s()],B.prototype,`collectionsLoading`,void 0),g([s()],B.prototype,`mediaCountLoading`,void 0),g([s()],B.prototype,`announcementsLoading`,void 0),g([s()],B.prototype,`heroBlockOrientation`,void 0),g([s()],B.prototype,`mediaCounts`,void 0),g([s()],B.prototype,`waybackPageCount`,void 0),g([s()],B.prototype,`announcements`,void 0),g([s()],B.prototype,`homePageServices`,void 0),g([s()],B.prototype,`currentPage`,void 0),g([s()],B.prototype,`lastPageReached`,void 0),g([s()],B.prototype,`navBaseUrl`,void 0),g([s()],B.prototype,`apiBaseUrl`,void 0),g([s()],B.prototype,`isPrerendering`,void 0),g([s()],B.prototype,`abtestManager`,void 0),g([s()],B.prototype,`waybackSearchHandler`,void 0),g([s()],B.prototype,`tvSearchHandler`,void 0),g([s()],B.prototype,`shouldShowOnboardingCarousel`,void 0),g([s()],B.prototype,`heroLogoSrc`,void 0),g([s()],B.prototype,`heroCaption`,void 0),g([s()],B.prototype,`heroLinkUrl`,void 0),g([s()],B.prototype,`heroLinkAriaLabel`,void 0),g([d(`#page-container`)],B.prototype,`pageContainer`,void 0),g([d(`infinite-scroller`)],B.prototype,`infiniteScroller`,void 0),B=g([u(`home-page`),i()],B);var V=B;export{V as default};