const __vite__mapDeps=(i,m=__vite__mapDeps,d=(m.f||(m.f=["offshoot_assets/default-content-BDXVbXgr.js","offshoot_assets/vendor-lit-MQ4DzmMc.js","offshoot_assets/content-template-MAs9DXX8.js","offshoot_assets/openlibrary-content-Dso2MkXb.js","offshoot_assets/wayback-content-DBQSXTzz.js","offshoot_assets/braintree-endpoint-manager-Cl4r1ZaR.js","offshoot_assets/rolldown-runtime-QTnfLwEv.js","offshoot_assets/dist-DuoWfHPQ.js","offshoot_assets/index-NFRVgqV2.js","offshoot_assets/vendor-shared-62wh2Xkx.js","offshoot_assets/vendor-topnav-Dcg-INpV.js","offshoot_assets/analytics-handler-DK_L78yH.js","offshoot_assets/vendor-router-BJcn_WPh.js","offshoot_assets/dist-BqHmxVdQ.js","offshoot_assets/dist-CjVcNP8S.js","offshoot_assets/is-archive-org-page-BUcF8U8h.js"])))=>i.map(i=>d[i]);
import{A as e,C as t,E as n,N as r,O as i,P as a,S as o,T as s,l as c,v as l,w as u,x as d}from"./vendor-lit-MQ4DzmMc.js";import{_ as f,f as p,h as m,t as h}from"./index-NFRVgqV2.js";import"./dist-DuoWfHPQ.js";import{t as g}from"./content-template-MAs9DXX8.js";var _=class{parseTemplate(e){let t=e.template;return t=this.replaceSalutation(t,e.displayName),t}replaceSalutation(e,t){let n=/({{username\|?(.*?)}})/,r=e.match(n)??[];return e.replace(n,t??r[2])}},v=`/offshoot_assets/brewster_kahle-Dw7-QZTt.jpg`,y=`/offshoot_assets/openlibrary-logo-jPfZSlNH.svg`,b=`/offshoot_assets/wayback-logo-square-CYuJrg76.svg`,x=class extends s{constructor(...e){super(...e),this.contentParser=new _}updated(e){(e.has(`displayName`)||e.has(`contentTemplate`))&&this.updateAppealLetterRenderedContent()}render(){return i`
      ${this.displayVideo?this.videoTemplate:n}
      ${this.headlineTemplate} ${this.primaryImageTemplate}
      ${this.renderedContent} ${this.footerTemplate}
    `}get videoTemplate(){return i`
      <figure id="header-video">
        <iframe
          src="https://archive.org/embed/wayback-machine-1996"
          width="100%"
          height="480"
          frameborder="0"
          webkitallowfullscreen="true"
          mozallowfullscreen="true"
          allowfullscreen
        ></iframe>
        <figcaption>
          ${a(`The Wayback Machine's First Crawl, 1996`)}
        </figcaption>
      </figure>
    `}get headlineTemplate(){let e=this.contentTemplate?.options?.headline;return e?i`
      <h1>${e}</h1>
      <hr />
    `:i``}get footerTemplate(){return i`
      <p class="tax-info">
        ${a(i`Please make a tax-deductible donation now.<br />You will receive
            an email confirmation that can be used for your tax records.<br />The
            Internet Archive is a California non-profit charity that is
            tax-exempt under section 501c3 of the Internal Revenue Code. Federal
            Tax ID Number 94-3242767.`)}
      </p>
    `}get primaryImageTemplate(){let e=this.primaryImageSource;if(!e)return i``;let t=this.contentTemplate?.options,r=t?.primaryImageAltText,a=t?.primaryImageCaption;return i`
      <div id="primary-image">
        <figure>
          <img src=${e} alt=${l(r)} lazy />
          ${a?i`<figcaption>${a}</figcaption>`:n}
        </figure>
      </div>
    `}get primaryImageSource(){let e;switch(this.contentTemplate?.options?.primaryImage){case g.brewster:e=v;break;case g.wayback:e=b;break;case g.openlibrary:e=y;break}return e}updateAppealLetterRenderedContent(){let e=this.contentTemplate?.template;if(!e||!this.contentParser)return;let t=this.contentParser.parseTemplate({template:e,displayName:this.displayName});this.renderedContent=i` ${c(t)} `,this.dispatchEvent(new Event(`contentParsed`))}static get styles(){return e`
      h1 {
        margin: 2rem;
      }

      hr {
        border: 0;
        border-top: 1px solid #d9d9d9;
        margin-bottom: 3rem;
      }

      p,
      ul {
        font-size: 1.4rem;
        line-height: 1.5;
        margin: 1rem 2rem;
      }

      .tax-info {
        font-style: italic;
      }

      #primary-image {
        float: right;
      }

      figure {
        margin: 0 2rem;
      }

      figure img {
        height: 150px;
      }

      figcaption {
        font-style: italic;
        text-align: center;
      }

      #header-video figcaption {
        font-size: 1.2rem;
      }
    `}};h([t({type:String})],x.prototype,`displayName`,void 0),h([t({type:Object})],x.prototype,`contentTemplate`,void 0),h([t({type:Boolean})],x.prototype,`displayVideo`,void 0),h([t({type:Object})],x.prototype,`contentParser`,void 0),h([o()],x.prototype,`renderedContent`,void 0),x=h([u(`donate-page-appeal-letter`),r()],x);var S=`/offshoot_assets/cryptocurrency_icons-YRIMQB7N.png`,C=class extends s{render(){return i`
      <a href="/donate/cryptocurrency/" class="wrapper">
        <div class="content-stack">
          <div class="button-text">${a(`Donate cryptocurrency`)}</div>
          <img
            class="crypto-icons"
            src=${S}
            alt=${a(`Cryptocurrency icons`)}
            lazy
          />
        </div>
      </a>
    `}static get styles(){return e`
      .wrapper {
        height: 60px;
        border: 1px solid #333;
        border-radius: 5px;
        display: flex;
        align-items: center;
        justify-content: center;
        color: inherit;
        text-decoration: none;
      }

      .content-stack {
        text-align: center;
      }

      .button-text {
        font-size: 18px;
        font-weight: bold;
      }

      .crypto-icons {
        height: 16px;
      }
    `}};C=h([u(`cryptocurrency-button`),r()],C);var w=class extends s{constructor(...e){super(...e),this.resizeObserverInitialized=!1,this.userFetched=!1,this.refererParam=null,this.originParam=null,this.platformParam=null,this.ymdParam=null,this.videoABTestTracked=!1,this.donationFormOriginSet=!1}get appealLetterContent(){if(!this.appealLetterContentFn)return;let e=p();return this.cachedAppealLetterContent?.locale!==e&&(this.cachedAppealLetterContent={locale:e,content:this.appealLetterContentFn()}),this.cachedAppealLetterContent.content}render(){return i`
      <div id="page-container">
        <div
          id="page-content-container"
          class="${this.useFullWidth?`full-width`:``}"
        >
          ${this.mainTemplateDependenciesReady?this.mainTemplate:n}
        </div>
      </div>
    `}get mainTemplateDependenciesReady(){let e=this.appealLetterContentFn!==void 0,t=this.videoVisible!==void 0;return this.resizeObserverInitialized&&this.userFetched&&e&&t}get mainTemplate(){return i`
      <div
        id="column-container"
        class="
          ${this.appealVisible?`appeal-visible`:`appeal-hidden`}
          ${this.isSmallScreen?`small-screen`:``}
        "
      >
        <div class="column appeal-letter-column">
          <donate-page-appeal-letter
            ?displayVideo=${this.videoVisible}
            .contentTemplate=${this.appealLetterContent}
            .displayName=${this.user?.screenname}
          >
          </donate-page-appeal-letter>
        </div>

        <div class="column donation-form-column">
          <div class="content-container">
            <div class="donation-form-container">
              <slot name="donation-form-slot"></slot>
            </div>

            <div class="crypto-donate">
              <p class="or-separator">
                <span>${a(`or`)}</span>
              </p>

              <cryptocurrency-button></cryptocurrency-button>

              <p>
                ${a(i`Questions? Visit our
                    <a
                      href="https://help.archive.org/help/category/donations-2/"
                      >Donation FAQs</a
                    >.`)}
              </p>
            </div>

            <div class="other-methods">
              <h2>${a(`Other ways to donate`)}</h2>
              <p>
                <strong>${a(`Checks (US Currency Only):`)}</strong><br />
                ${a(`Mail your donation to:`)}
              </p>
              <p>
                Internet Archive<br />
                C/O Philanthropy Department<br />
                P.O. Box 45148<br />
                San Francisco, California&nbsp;&nbsp;94145-0148<br />
              </p>
              <p>
                ${a(`In order to ensure you receive an acknowledgement of your gift as quickly as possible, please include an email address with your mailed donation.`)}
                <strong>
                  ${a(`We regret that we cannot accept cash donations.`)}
                </strong>
              </p>
              <p>
                <strong>${a(`Stock or Wire Transfer:`)}</strong><br />
                ${a(i`If you would like to make a stock or wire transfer gift,
                    please reach out to Joy Chesbrough, Director of
                    Philanthropy, at
                    <a href="mailto:donations@archive.org"
                      >donations@archive.org</a
                    >`)}
              </p>
            </div>
            <div class="donor-emails">
              ${a(i`By donating, you will receive occasional donor-related
                emails from the Internet Archive. We do not sell or trade your
                information with anyone. You can unsubscribe at any time.`)}
            </div>
          </div>
        </div>
      </div>

      <div class="tax-info">
        ${a(i`The Internet Archive is a US 501c3 non-profit organization. Our
          tax ID number is 94-3242767. Your gift is tax deductible to the full
          extent provided by the law.`)}
      </div>
    `}updated(e){e.has(`appServices`)&&(this.setupAppServicesDependents(),this.sendDonationFlowEvents(`referer`),this.sendDonationFlowEvents(`platform`),this.sendDonationFlowEvents(`origin`)),(e.has(`appealVisible`)||e.has(`videoABTestActive`))&&(this.setDonationFormOrigin(),this.trackVideoABTest())}disconnectedCallback(){this.shutdown()}firstUpdated(){let e=new URL(window.location.href);this.refererParam=e.searchParams.get(`referer`),this.originParam=e.searchParams.get(`origin`),this.platformParam=e.searchParams.get(`platform`),this.ymdParam=e.searchParams.get(`ymd`)}async sendDonationFlowEvents(e){let t=await(await this.appServices?.get())?.analyticsHandler.get(),n=new URL(window.location.href),r=``;switch(e){case`platform`:r=n.searchParams.get(`platform`)||`ia`;break;case`referer`:r=n.searchParams.get(`referer`)||document.referrer||`unset`;break;case`origin`:r=n.searchParams.get(`origin`)||`unset`;break}t?.sendEvent({category:`DonationFlow`,action:e,label:r})}checkAndApplyCustomDonateFormStyling(){if(!this.donationFormController)return;let e=new URL(window.location.href),t=e.searchParams.get(`selectedFillColor`)||``,n=e.searchParams.get(`selectedTextColor`)||``;t&&this.donationFormController?.style.setProperty(`--donateFormSelectedOptionBgColor`,decodeURIComponent(t)),n&&this.donationFormController?.style.setProperty(`--donateFormSelectedOptionTextColor`,decodeURIComponent(n))}handleResize(e){let t=e.contentRect.width,n=t+(this.appRootController?.getWindowScrollbarWidth()??0);switch(this.isSmallScreen=n<=350,e.target){case this.pageContainer:this.resizeObserverInitialized||(this.resizeObserverInitialized=!0),this.useFullWidth=t<=600,this.appealVisible=n>=800;break;default:break}}async setupAppServicesDependents(){await Promise.all([this.sendPageViewedEvent(),this.setupPaypalUpsellSlot(),this.setupResizeObserver(),this.setupTopNavAndFooter(),this.setupPageMetadata(),this.setupLocalizationChangeHandler(),this.fetchUser(),this.setupVideoABTest(),this.fetchAppealTemplate()]),await this.setupDonationForm()}async setupPaypalUpsellSlot(){await(await(await this.appServices?.get())?.lightDOMTunnel.get())?.setSlotContent(`paypal-upsell-button`,i` <div id="paypal-upsell-button"></div> `)}async fetchAppealTemplate(){this.linkedFromWayback?this.setWaybackLetter():this.linkedFromOpenLibrary?this.setOpenLibraryLetter():this.setDefaultLetter()}async setDefaultLetter(){let e=await f(()=>m(()=>import(`./default-content-BDXVbXgr.js`),__vite__mapDeps([0,1,2])));this.appealLetterContentFn=e.defaultContent}async setOpenLibraryLetter(){let e=await f(()=>m(()=>import(`./openlibrary-content-Dso2MkXb.js`),__vite__mapDeps([3,1,2])));this.appealLetterContentFn=e.openLibraryContent}async setWaybackLetter(){let e=await f(()=>m(()=>import(`./wayback-content-DBQSXTzz.js`),__vite__mapDeps([4,1,2])));this.appealLetterContentFn=e.waybackContent}get linkedFromOpenLibrary(){return this.platformParam===`ol`||(this.originParam?.includes(`olwww`)??!1)}get linkedFromWayback(){return this.platformParam===`wb`||(this.originParam?.includes(`wbwww`)??!1)}get linkedFromWaybackOrOpenLibrary(){return this.linkedFromOpenLibrary||this.linkedFromWayback}get videoABTestTracking(){let e;return this.videoABTestActive&&this.appealVisible&&(e=this.videoVisible?`PageVariant:IAVideo`:`PageVariant:IANoVideo`),e}async setupVideoABTest(){if(this.linkedFromWaybackOrOpenLibrary){this.videoVisible=!1,this.videoABTestActive=!1;return}let e=await(await(await this.appServices?.get())?.abtestManager.get())?.variantFor(`DonatePageVideo`);this.videoABTestActive=e!==void 0,this.videoVisible=e===`On`}async trackVideoABTest(){!this.videoABTestActive||!this.appealVisible||this.videoABTestTracked||((await(await this.appServices?.get())?.analyticsHandler.get())?.sendEvent({category:`DonatePage`,action:`IAVideoABTestViewed`,label:this.videoABTestTracking}),this.videoABTestTracked=!0)}async setupPageMetadata(){let e=await(await this.appServices?.get())?.pageMetadataService.get(),t=a(`Donate to the Internet Archive!`);e?.setTitle(t),e?.addLink({rel:`canonical`,path:`/donate`}),e?.setMetaTag({property:`og:type`,content:`website`}),e?.setMetaTag({property:`og:url`,content:`https://archive.org/donate?iax=ctx|facebook&origin=og-facebook`}),e?.setMetaTag({property:`og:image`,content:`https://archive.org/images/brewster_kahle-lg.jpg`}),e?.setMetaTag({property:`og:image:width`,content:`1700`}),e?.setMetaTag({property:`og:image:height`,content:`1700`});let n=a(`Our job is getting bigger. What we learned in the last year: we need
      good information online. Reliable information. Websites and data that
      don't disappear and can't be changed after-the-fact.`);e?.setMetaTag({property:`og:description`,content:n})}async setupLocalizationChangeHandler(){let e=await(await this.appServices?.get())?.localizationManager.get();this.localizationManagerUnsubscribe=e?.on(`languageChanged`,()=>{this.setupPageMetadata()})}async fetchUser(){let e=await this.appServices?.get(),t=await e?.appRootController.get();t?.showActivityIndicator(!0);try{let t=await(await e?.userService.get())?.getLoggedInUser();this.user=t?.success}catch{console.error(`There was an error fetching the user`)}return t?.showActivityIndicator(!1),this.userFetched=!0,this.user}shutdown(){this.disconnectResizeObserver(),this.localizationManagerUnsubscribe&&this.localizationManagerUnsubscribe()}disconnectResizeObserver(){this.resizeObserver?.removeObserver({handler:this,target:this.pageContainer})}async setupResizeObserver(){this.disconnectResizeObserver();let e=await this.appServices?.get(),t=await e?.appRootController.get();this.appRootController=t;let n=await e?.resizeObserver.get();this.resizeObserver=n,n?.addObserver({handler:this,target:this.pageContainer})}async setupTopNavAndFooter(){(await(await this.appServices?.get())?.appRootController.get())?.setAppFooterMode(`off`)}async sendPageViewedEvent(){(await(await this.appServices?.get())?.analyticsHandler.get())?.sendEvent({category:`DonatePage`,action:`pageViewed`})}async addDonationFormToDOM(){if(this.donationFormController)return;let t=await this.appServices?.get(),n=await t?.appConfig.get(),r=await t?.lightDOMTunnel.get(),a=n?.isPrerendering,o,s,c,u,d;if(!a){o=n?.environment;let e=n?.donationForm;s=e?.braintreeAuthToken,c=n?.recaptchaSiteKey,u=e?.venmoProfileId,d=e?.googlepayMerchantId}let f=new URL(window.location.href).searchParams.get(`bannerTemplate`),p=f?`bannerTemplate-${f}`:void 0;await r?.setSlotContent(`donation-form-slot`,i`
        <donation-form-controller
          class=${l(p)}
          environment=${l(o)}
          braintreeauthtoken=${l(s)}
          recaptchasitekey=${l(c)}
          venmoprofileid=${l(u)}
          googlepaymerchantid=${l(d)}
          .donorEmail=${this.user?.username??``}
        >
        </donation-form-controller>
        <div id="recaptcha"></div>
      `),await r?.setStyle(`donation-form-style`,e`
        donation-form-controller.bannerTemplate-chevron {
          --paymentButtonSelectedColor: #194880;
          --paymentButtonFontColor: #000;
          --paymentButtonSelectedFontColor: #fff;
        }
      `),await r?.setStyle(`recaptcha-style`,e`
        #recaptcha {
          z-index: 1;
          position: absolute;
        }
      `)}get donationFormController(){return document.querySelector(`donation-form-controller`)}async setupDonationForm(){await this.addDonationFormToDOM();let e=this.donationFormController;if(!e)return;let t=await this.appServices?.get(),n=await t?.lazyLoaderService.get();n&&(e.lazyLoaderService=n),e.recaptchaElement=document.querySelector(`#recaptcha`);let r=await t?.appConfig.get();e.endpointManager=new(await(f(()=>m(()=>import(`./braintree-endpoint-manager-Cl4r1ZaR.js`),__vite__mapDeps([5,6,7,8,9,10,1,11,12,13,14,15]))))).BraintreeEndpointManager({apiUrl:r?.archiveApiUrl??``,documentReferrer:r?.documentReferrer??``}),e.analyticsHandler=await t?.analyticsHandler.get(),e.modalManager=await t?.modalManager.get();let i=(await(await t?.userService.get())?.getLoggedInUser())?.success;i&&(e.loggedInUser=i.screenname);let a=this.refererParam??r?.documentReferrer;e.referrer=this.getDonationFormReferrer({referer:a}),e.addEventListener(`paymentProvidersHostedFieldsRetry`,async e=>{let n=e.detail.retryNumber;(await t?.analyticsHandler.get())?.sendEvent({category:`DonationFormAnalytics`,action:`HostedFields Retry: ${n}`})}),e.addEventListener(`paymentProvidersHostedFieldsFailed`,async()=>{(await t?.analyticsHandler.get())?.sendEvent({category:`DonationFormAnalytics`,action:`HostedFields Failed to Load`})}),this.setDonationFormOrigin(),this.checkAndApplyCustomDonateFormStyling()}async setDonationFormOrigin(){let e=this.donationFormController;if(!e||this.donationFormOriginSet&&this.videoABTestTracked)return;this.donationFormOriginSet=!0;let t=this.originParam,n=this.videoABTestTracking,r=[];t&&r.push(t),n&&r.push(n),e.origin=r.join(`___`)}getDonationFormReferrer(e){let t={};t.ctx=`page`,t.uid=this.user?.userid??``;let n=e?.referer;return n&&(t.ref=encodeURIComponent(n)),Object.keys(t).map(e=>`${e}=${t[e]}`).join(`;`)}static get styles(){return e`
      :host {
        display: block;
      }

      #page-container {
        max-width: var(--app-page-max-width);
        margin: auto;
      }

      #page-content-container {
        margin: 1rem 2rem 2rem 2rem;
      }

      #page-content-container.full-width {
        margin-left: 0;
        margin-right: 0;
      }

      #column-container {
        display: flex;
        align-items: flex-start;
      }

      a:link,
      a:visited {
        text-decoration: none;
        color: var(--ia-theme-link-color);
      }

      a:hover {
        text-decoration: underline;
      }

      .column {
        background-color: var(--ia-theme-secondary-background-color, #fff);
      }

      #column-container.appeal-hidden .appeal-letter-column {
        display: none;
      }

      .appeal-letter-column {
        margin-right: 2rem;
        flex: 1;
        padding-bottom: 2rem;
        padding-top: 2rem;
      }

      #column-container.appeal-hidden .donation-form-column {
        flex: 1;
        max-width: 72rem;
      }

      .donation-form-column {
        flex: 0;
        margin: 0 auto;
        padding: 3rem 1rem 2rem 1rem;
      }

      .small-screen .donation-form-column {
        padding-left: 0.25rem;
        padding-right: 0.25rem;
      }

      .donation-form-column .content-container {
        margin: auto;
        width: 32rem;
      }

      .small-screen .donation-form-column .content-container {
        width: 30rem;
      }

      .donation-form-container {
        --formSectionBadgeBackgroundColor: #2c2c2c;
        --formSectionContentBackgroundColor: var(
          --ia-theme-secondary-background-color
        );
        --paymentButtonColor: var(--ia-theme-secondary-background-color);
      }

      .crypto-donate {
        margin-left: 3.4rem;
        text-align: center;
        font-size: 1.4rem;
      }

      .or-separator {
        position: relative;
        margin: 0 4rem;
        font-size: 2.6rem;
        font-weight: 700;
        text-transform: uppercase;
        text-align: center;
      }

      .or-separator:before {
        position: absolute;
        top: 49%;
        right: 0;
        left: 0;
        height: 2px;
        content: '';
        background: #333;
      }

      .or-separator span {
        display: inline-block;
        position: relative;
        padding: 1rem;
        background: #fff;
      }

      .other-methods {
        font-size: 1.4rem;
      }

      .other-methods p {
        line-height: 1.4;
      }

      .other-methods h2 {
        font-size: 3rem;
        font-weight: 500;
        margin-bottom: 1rem;
        margin-top: 2rem;
      }

      .donor-emails {
        font-size: 1.1rem;
        line-height: 1.6;
      }

      .tax-info {
        max-width: 50rem;
        font-size: 1.4rem;
        text-align: center;
        line-height: 1.5;
        margin: 3rem auto 2rem auto;
      }
    `}};h([t({type:Object})],w.prototype,`appServices`,void 0),h([o()],w.prototype,`appealVisible`,void 0),h([o()],w.prototype,`useFullWidth`,void 0),h([o()],w.prototype,`isSmallScreen`,void 0),h([o()],w.prototype,`resizeObserverInitialized`,void 0),h([o()],w.prototype,`user`,void 0),h([o()],w.prototype,`userFetched`,void 0),h([o()],w.prototype,`appealLetterContentFn`,void 0),h([o()],w.prototype,`videoVisible`,void 0),h([o()],w.prototype,`videoABTestActive`,void 0),h([d(`#page-container`)],w.prototype,`pageContainer`,void 0),w=h([u(`donate-page`),r()],w);var T=w;export{T as default};