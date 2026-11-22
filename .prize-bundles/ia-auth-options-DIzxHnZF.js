import{A as e,C as t,E as n,F as r,N as i,O as a,P as o,S as s,T as c,_ as l,b as u,k as d,l as ee,u as f,v as p,w as m,x as h}from"./vendor-lit-MQ4DzmMc.js";import{s as g}from"./vendor-shared-62wh2Xkx.js";import{r as _,t as v}from"./vendor-topnav-Dcg-INpV.js";import{a as te,t as y}from"./index-NFRVgqV2.js";import{i as b}from"./dist-BqHmxVdQ.js";import"./ia-status-indicator-Bd61y7bd.js";import{n as x}from"./dist-CjVcNP8S.js";import{t as S}from"./log-Bvw5tZC3.js";import{t as C}from"./third-party-auth-service-C371OWBh.js";import{t as w}from"./purify.es-RmFZhtLI.js";import"./ia-button-DYuVUpnO.js";import{t as T}from"./ia-logo-gray-circle-BADeBCkf.js";import{t as E}from"./email-link-DbmrSSqK.js";import"./ia-icon-ClrHaxRi.js";var D=d`
  <svg
    height="16"
    viewBox="0 0 22 16"
    width="22"
    xmlns="http://www.w3.org/2000/svg"
    id="eye-crossed"
  >
    <g fill="none" fill-rule="evenodd">
      <g stroke="#333" stroke-width="1.5" transform="translate(1 2)">
        <path 
          d="m0 6c4.44444444-4 7.77777778-6 10-6 2.2222222 0 5.5555556 2 10 6-4.4444444 4-7.7777778 6-10 6-2.22222222 0-5.55555556-2-10-6z" stroke-linejoin="round"/>
        <circle cx="10" cy="6" r="5"/>
      </g>
      <circle cx="11" cy="8" fill="#333" r="2"/>
      <path d="m4.5 1.5 13.5 13.5" stroke="#333" stroke-linecap="round" stroke-width="2"/>
    </g>
  </svg>`,O=d`
  <svg 
    height="16" 
    viewBox="0 0 22 16" 
    width="22" 
    xmlns="http://www.w3.org/2000/svg"
    id="eye-open"
  >
    <g fill="none" fill-rule="evenodd">
      <g stroke="#333" stroke-width="1.5" transform="translate(1 2)">
        <path 
          d="m0 6c4.44444444-4 7.77777778-6 10-6 2.2222222 0 5.5555556 2 10 6-4.4444444 4-7.7777778 6-10 6-2.22222222 0-5.55555556-2-10-6z" 
          stroke-linejoin="round"
        />
        <circle cx="10" cy="6" r="5"/>
      </g>
      <circle cx="11" cy="8" fill="#333" r="2"/>
    </g>
  </svg>`,k=class extends c{constructor(...e){super(...e),this.passwordVisible=!1}render(){return this.passwordInput?a`
      <button
        type="button"
        class="password-icon"
        title=${o(r`${this.passwordVisible?`Hide password`:`Show password`}`)}
        @click=${this.togglePasswordVisibility}
      >
        ${this.passwordVisible?O:D}
      </button>
    `:n}willUpdate(e){e.has(`passwordInput`)&&this.passwordInput&&(this.passwordVisible=this.passwordInput.type===`text`)}togglePasswordVisibility(){this.passwordInput&&(this.passwordVisible=!this.passwordVisible,this.passwordInput.type=this.passwordVisible?`text`:`password`)}static get styles(){return e`
      .password-icon {
        background: transparent;
        vertical-align: middle;
        padding: 7px 4px;
        cursor: pointer;
        border: 0;
      }

      .password-icon svg {
        margin-top: 2px;
      }
    `}};y([t({type:Object})],k.prototype,`passwordInput`,void 0),y([s()],k.prototype,`passwordVisible`,void 0),k=y([m(`ia-password-toggle`),i()],k);var A=class extends c{constructor(...e){super(...e),this.dividerText=o(`Or`)}render(){return a`
      <div class="line"></div>
      ${this.dividerText}
      <div class="line"></div>
    `}static get styles(){return e`
      :host {
        font-size: 1.8rem;
        font-weight: 700;
        width: var(--textDividerWidth, 100%);
        display: flex;
        flex-direction: row;
        align-items: center;
        justify-content: center;
        gap: 4%;
      }

      .line {
        height: 0.2rem;
        background-color: #000;
        width: 45%;
      }
    `}};y([t({type:String})],A.prototype,`dividerText`,void 0),A=y([m(`ia-text-divider`),i()],A);var j=class{constructor(){this.email=``,this.screenname=``}};function M(){return{GENERIC:o(`Something happened when we tried to connect to your preferred login.  You may try again.`),VERIFICATION_FAIL:o(`We could not verify your account.  You may try again.`),UNLINK_ERROR:o(`We could not unlink your account.  You may try again.`),POPUP_BLOCKED:o(`Your browser has blocked the Google pop-up. Please allow & retry again.`),GOOGLE_ACCOUNT_MISMATCH:o(`Your Google account is not associated with this Internet Archive account. Please ensure you are using the correct Google account.`)}}function N(){return{GENERIC_WAIT:o(`Waiting on provider...`),LOCATING_ACCOUNT:o(`Locating your account...`),LOGGING_IN:o(`Logging you in...`),REDIRECT_HOME:o(`Redirecting you home...`),REDIRECT_BACK:o(`Redirecting you back...`),REGISTER_ACCOUNT:o(`Registering your account...`),CONNECT_ACCOUNT:o(`Connecting your account...`)}}var P=class extends c{constructor(...e){super(...e),this.selectedMailingLists=[]}setMailingList(e){let t=e.target,n=t.name;this.dispatchEvent(new CustomEvent(`selectedMailingList`,{detail:{listName:n,checked:t.checked}}))}render(){return this.mailingLists?a`
      ${Object.entries(this.mailingLists).map(([,e])=>e.public?a`
          <div class="element">
            <input
              type="checkbox"
              id="${e.key}"
              name="${e.key}"
              @click=${this.setMailingList}
              .checked=${this.selectedMailingLists.includes(e.key)}
            />
            <label for="${e.key}"> ${e.name}: ${e.short_desc} </label>
          </div>
        `:n)}
    `:n}};y([t({type:Object})],P.prototype,`mailingLists`,void 0),y([t({type:Array})],P.prototype,`selectedMailingLists`,void 0),P=y([m(`mailing-lists`)],P);var F={ml_best_of:{interest_id:`e7594f8604`,key:`ml_best_of`,name:`Best of the Archive`,public:!0,selected_by_default:!1,short_desc:`Useful resources, unique stories, and fun finds from our collections`},ml_donors:{interest_id:`ff44b81002`,key:`ml_donors`,name:`Donor Communications`,public:!1,selected_by_default:!1,short_desc:`Information about how your donations are being used.`},ml_events:{interest_id:`98cc7bf635`,key:`ml_events`,name:`Event Notices`,public:!0,selected_by_default:!1,short_desc:`Invitations to and news about our events`}},I=class extends c{constructor(...e){super(...e),this.mailingListPreferences={ml_best_of:!1,ml_events:!1}}render(){return a`
      <section class="header">
        <img src=${T} alt=${o(`Internet Archive Logo`)} />
        <h2>${o(`Welcome to the Internet Archive`)}</h2>
        <p>${o(`We hope you'll explore all we have to offer.`)}</p>
      </section>
      <h4>${o(`Email address`)}</h4>
      <p class="email">${this.email??o(`N/A`)}</p>
      <form @submit=${this.handleSubmit}>
        <h4>${o(`Announcements`)}</h4>
        <p>
          ${o(`Stay up to date with what’s happening at the Internet Archive by signing up for our free newsletters.`)}
        </p>
        <mailing-lists
          .mailingLists=${F}
          @selectedMailingList=${this.updateMailingLists}
        >
        </mailing-lists>
        <ia-button
          mode="primary"
          type="submit"
          ?loading=${this.isProcessing}
          .loadingText=${this.progressMessage??``}
        >
          ${o(`Continue`)}
        </ia-button>
      </form>
    `}updateMailingLists(e){let t=e.detail?.listName;!t||this.mailingListPreferences[t]===void 0||(this.mailingListPreferences[e.detail?.listName]=e.detail?.checked)}handleSubmit(e){e.preventDefault();let t=Object.keys(this.mailingListPreferences).filter(e=>this.mailingListPreferences[e]===!0).join(`,`);this.dispatchEvent(new CustomEvent(`registrationFormSubmitted`,{detail:{notifications:t}}))}static get styles(){return e`
      img {
        height: 8rem;
        margin: 3rem 0;
        vertical-align: middle;
      }

      .header {
        text-align: center;
      }

      h2 {
        font-size: 2.4rem;
        margin-bottom: 0.6rem;
        margin: 0 0 0.6rem;
      }

      h4 {
        margin-bottom: 0;
      }

      p {
        margin-top: 0;
      }

      ia-button {
        margin-top: 2rem;
      }
    `}};y([t({type:String})],I.prototype,`email`,void 0),y([t({type:Boolean})],I.prototype,`isProcessing`,void 0),y([t({type:String})],I.prototype,`progressMessage`,void 0),y([s()],I.prototype,`mailingListPreferences`,void 0),I=y([m(`ia-new-user-registration`),i()],I);var L=class extends c{constructor(...e){super(...e),this.mailingListPreferences={ml_best_of:!1,ml_events:!1}}render(){return a`
      <section class="header">
        <img src=${T} alt=${o(`Internet Archive Logo`)} />
        <h2>${o(`Found existing account`)}</h2>
        <p>
          ${o(`We found an existing account that matches your information. We'll link these.`)}
        </p>
      </section>
      <section class="account-info">
        <h4>${o(`Email address`)}</h4>
        <p>${this.email??o(`N/A`)}</p>
        <h4>
          ${o(`Screen name`)}<span class="aside"></span>
           ${o(`(this is how others will see you)`)}</span
          >
        </h4>
        <p>${this.screenname??`N/A`}</p>
      </section>
      <div class="cta">
        <ia-button
          mode="primary"
          class="confirm"
          ?loading=${this.isProcessing}
          .loadingText=${this.progressMessage??o(`Linking...`)}
          @click=${()=>this.dispatchEvent(new Event(`linkAccountConfirmed`))}
        >
          ${o(`Link accounts`)}
        </ia-button>
        <ia-button
          mode="danger"
          class="cancel"
          @click=${()=>this.dispatchEvent(new Event(`linkAccountCanceled`))}
        >
          ${o(`Cancel`)}
        </ia-button>
      </div>
    `}static get styles(){return e`
      img {
        height: 8rem;
        margin: 3rem 0;
        vertical-align: middle;
      }

      .header {
        text-align: center;
      }

      h2 {
        font-size: 2.4rem;
        margin-bottom: 0.6rem;
        margin: 0 0 0.6rem;
      }

      h4 {
        margin-bottom: 0;
      }

      .aside {
        font-weight: 400;
      }

      p {
        margin-top: 0;
      }

      .cta {
        display: inline-flex;
        gap: 0.5rem;
      }
      ia-button {
        margin-top: 2rem;
      }
    `}};y([t({type:String})],L.prototype,`email`,void 0),y([t({type:String})],L.prototype,`screenname`,void 0),y([t({type:Boolean})],L.prototype,`isProcessing`,void 0),y([t({type:String})],L.prototype,`progressMessage`,void 0),y([s()],L.prototype,`mailingListPreferences`,void 0),L=y([m(`ia-link-account-prompt`),i()],L);var R=class extends Error{};R.prototype.name=`InvalidTokenError`;function z(e){return decodeURIComponent(atob(e).replace(/(.)/g,(e,t)=>{let n=t.charCodeAt(0).toString(16).toUpperCase();return n.length<2&&(n=`0`+n),`%`+n}))}function B(e){let t=e.replace(/-/g,`+`).replace(/_/g,`/`);switch(t.length%4){case 0:break;case 2:t+=`==`;break;case 3:t+=`=`;break;default:throw Error(`base64 string is not of the correct length`)}try{return z(t)}catch{return atob(t)}}function V(e,t){if(typeof e!=`string`)throw new R(`Invalid token specified: must be a string`);t||(t={});let n=t.header===!0?0:1,r=e.split(`.`)[n];if(typeof r!=`string`)throw new R(`Invalid token specified: missing part #${n+1}`);let i;try{i=B(r)}catch(e){throw new R(`Invalid token specified: invalid base64 for part #${n+1} (${e.message})`)}try{return JSON.parse(i)}catch(e){throw new R(`Invalid token specified: invalid json for part #${n+1} (${e.message})`)}}var H=class extends c{constructor(...e){super(...e),this.clientId=``,this.csrfToken=``,this.archiveApiUrl=``}updated(e){e.has(`csrfToken`)&&this.loadGoogleAuth()}async loadGoogleAuth(){let e=()=>{this.handleAuthError({error:`The Google library failed to load. Please refresh and try again`,errorCode:`ga_init_timeout`,cannotUseProvider:!0})};new x().loadScript({src:`https://accounts.google.com/gsi/client`}).then(()=>this.initGoogle()).catch(()=>e())}intermediateIframeCloseCallback(e){S(`Google Auth intermediateIframeCloseCallback`,e),this.handleAuthError({error:`Intermediate iframe closed unexpectedly`})}initGoogle(){try{google.accounts.id.initialize({client_id:this.clientId,callback:e=>this.callback(e),allowed_parent_origin:`https://${this.archiveApiUrl}`,intermediateIframeCloseCallback:e=>this.intermediateIframeCloseCallback(e),nonce:this.csrfToken}),this.signinButton?google.accounts.id.renderButton(this.signinButton,{theme:`outline`,text:`signin_with`,size:`large`,logo_alignment:`left`,shape:`rectangular`,width:250}):console.error(`Google login button element not found.`)}catch(e){this.googleAuthInitError({error:`button_cannot_be_drawn`,details:e?.message||`Unknown error`})}}handleAuthError(e){let t=new CustomEvent(`handleAuthError`,{detail:e});this.dispatchEvent(t)}googleAuthInitError({error:e,details:t=`no_details_provided`}){let n=e===`idpiframe_initialization_failed`?`Please enable cookies if you wish to connect to Google.`:t;this.handleAuthError({error:n,errorCode:e,cannotUseProvider:!0,softMessage:!0})}googleProfileFetchFailure({error:e}){let t=e===`popup_blocked_by_browser`?M().POPUP_BLOCKED:M().GENERIC;this.handleAuthError({error:t,errorCode:e,cannotUseProvider:!0})}googleProfileReceived(e){this.googleProfile=e;let t=new j;t.email=this.googleProfile.email,t.screenname=this.googleProfile.name;let n=new CustomEvent(`profileReceived`,{detail:{tokenToClaim:this.googleAccount?.credential,thirdPartyProfile:t,provider:`google`}});this.dispatchEvent(n)}callback(e){if(!e){this.googleProfileFetchFailure({error:`no_google_user`});return}if(this.googleAccount=e,e.clientId!==this.clientId){this.googleProfileFetchFailure({error:`invalid_client_id`});return}let t;try{t=V(e.credential)}catch{this.googleProfileFetchFailure({error:`cannot_parse_profile`});return}this.googleProfileReceived(t)}render(){return a`
      <div>
        <!-- Placeholder for Google login button -->
        <div id="g_id_signin"></div>
      </div>
    `}};y([t({type:String})],H.prototype,`clientId`,void 0),y([t({type:String})],H.prototype,`csrfToken`,void 0),y([t({type:String})],H.prototype,`archiveApiUrl`,void 0),y([t({type:Object})],H.prototype,`googleAccount`,void 0),y([t({type:Object})],H.prototype,`googleProfile`,void 0),y([h(`#g_id_signin`)],H.prototype,`signinButton`,void 0),H=y([m(`ia-google-auth`)],H);var U=class extends c{constructor(...e){super(...e),this.archiveApiUrl=``,this.googleClientId=``,this.verifyUser=``,this.verificationEmail=``,this.csrfToken=``,this.linkedProviders=[],this.embedded=!1,this.currentOnboardPhase=`start`,this.isProcessing=!1,this.progressMessage=``,this.errorMessage=``,this.archiveAccount=null,this.thirdPartyProfile=null,this.tokenToClaim=``,this.provider=``,this.rawError={},this.iaUserToVerify=``}render(){return f(this.currentOnboardPhase,[[`start`,()=>this.startTemplate],[`register`,()=>this.registerUserTemplate],[`link`,()=>this.linkAccountTemplate]])??n}get startTemplate(){let e=this.errorMessage&&this.rawError?.softMessage?``:`error`;return a`
      <section>
        ${this.googleAuthButton}
        ${this.errorMessage?a`<p class=${e}>${this.errorMessage}</p>`:n}
      </section>
      <p>${this.progressMessage}</p>
    `}get registerUserTemplate(){return a`<ia-new-user-registration
      .email=${this.thirdPartyProfile?.email}
      .progressMessage=${this.progressMessage}
      ?isProcessing=${this.isProcessing}
      @registrationFormSubmitted=${e=>{this.sendAnalyticsEvent({action:`${this.provider}_onboard_registering`}),this.registerUser(e)}}
    ></ia-new-user-registration>`}get linkAccountTemplate(){return a`<ia-link-account-prompt
      .email=${this.archiveAccount?.email}
      .screenname=${this.archiveAccount?.screenname}
      .progressMessage=${this.progressMessage}
      ?isProcessing=${this.isProcessing}
      @linkAccountConfirmed=${()=>{this.sendAnalyticsEvent({action:`${this.provider}_onboard_linking`}),this.linkAccount()}}
      @linkAccountCanceled=${()=>{this.sendAnalyticsEvent({action:`${this.provider}_onboard_cancelLink`}),this.resetFormAndLightDom()}}
    ></ia-link-account-prompt>`}willUpdate(e){if(e.has(`this.verifyUser`)&&this.verifyUser&&(this.iaUserToVerify=this.verifyUser),e.has(`appServices`)&&this.appServices&&this.setupAppServices(),(e.has(`csrfToken`)||e.has(`fetchHandler`))&&this.setupThirdPartyAuthService(),e.has(`errorMessage`)&&this.errorMessage){let{errorCode:e=`not_found`}=this.rawError;this.sendAnalyticsEvent({action:`${this.provider}_error_${e}`})}e.has(`currentOnboardPhase`)&&this.provider&&(this.sendAnalyticsEvent({action:`${this.provider}_onboard_${this.currentOnboardPhase}`}),this.currentOnboardPhase===`verifiedLogin`&&this.dispatchEvent(new CustomEvent(`IAThirdPartyAuthVerified`,{detail:{email:this.archiveAccount?.email}})))}async setupAppServices(){await(await this.appServices?.get())?.fetchHandler.get().then(e=>{this.fetchHandler=e})}setupThirdPartyAuthService(){this.fetchHandler&&(this.thirdPartyAuthService=new C(this.csrfToken,this.fetchHandler))}sendAnalyticsEvent(e){S(`** auth analytic:`,e.action),this.analyticsHandler?.sendEventNoSampling({category:`Third-Party-Auth`,action:e.action,label:e.label})}resetAuthProps(){this.archiveAccount=null,this.tokenToClaim=``,this.isProcessing=!1,this.progressMessage=``,this.currentOnboardPhase=`start`}resetFormAndLightDom(){this.resetAuthProps(),this.endFormTakeover()}async getLinkedProvidersList(){S(`getLinkedProvidersList`);let e=await this.thirdPartyAuthService?.checkForLinkedProviders();this.linkedProviders=e?.providers}moveUserThroughOnboardFlow({account:e,linked:t,third_party_info:n}){let r=(e=`login`)=>{this.currentOnboardPhase=e,this.logUserIn()};this.iaUserToVerify?this.thirdPartyAuthService?.isAccountMatching(this.iaUserToVerify,{account:e,linked:t,third_party_info:n})?(r(`verifiedLogin`),this.getLinkedProvidersList()):this.handleAuthError({error:M().VERIFICATION_FAIL,errorCode:`ia_cant_verify`,cannotUseProvider:!0}):t?r(`linkedLogin`):(e&&n&&!t&&(this.requestFormTakeover(),this.currentOnboardPhase=`link`),e||(this.requestFormTakeover(),this.currentOnboardPhase=`register`))}async logUserIn(){this.progressMessage=N().LOGGING_IN,this.isProcessing=!0;try{let e=await this.thirdPartyAuthService?.logUserIn({provider:this.provider,providerToken:this.tokenToClaim,extraParams:this.embedded?{remember:`-1`}:{}});if(!e||e.error||!e.value||!e.success){this.handleAuthError({error:M().GENERIC,errorCode:`ia_cant_log_user_in`});return}let t=(e?.value).token;t&&this.dispatchEvent(new CustomEvent(`IAThirdPartyAuth:MGCToken`,{detail:{token:t}})),this.sendAnalyticsEvent({action:`${this.provider}_onboard_loggingIn`}),this.currentOnboardPhase=`verifiedLogin`}catch{this.handleAuthError({error:M().GENERIC,errorCode:`ia_cant_log_user_in`})}finally{this.isProcessing=!1}}async linkAccount(){this.progressMessage=N().CONNECT_ACCOUNT,this.isProcessing=!0;try{await this.thirdPartyAuthService?.linkUserToAccount({provider:this.provider,providerToken:this.tokenToClaim})&&await this.logUserIn()}catch{this.handleAuthError({error:M().UNLINK_ERROR})}finally{this.isProcessing=!1}}async registerUser(e){if(this.progressMessage=N().REGISTER_ACCOUNT,this.isProcessing=!0,!await this.thirdPartyAuthService?.registerUser({provider:this.provider,providerToken:this.tokenToClaim,extraParams:{notifications:e.detail.notifications}})){this.resetFormAndLightDom();return}this.isProcessing=!1,this.logUserIn()}async startConnectingUserToIA(e){this.errorMessage=``,S(`profile received`,e);let{tokenToClaim:t,thirdPartyProfile:n,provider:r}=e.detail;this.provider=r,this.thirdPartyProfile=n,this.tokenToClaim=t,this.progressMessage=N().LOCATING_ACCOUNT,this.isProcessing=!0;try{let e=await this.thirdPartyAuthService?.checkIfAccountExists({provider:this.provider,providerToken:this.tokenToClaim});if(this.isProcessing=!1,!e)return;let t=e.value?.account,n=new j;if(n.screenname=t?.screenname??``,n.email=t?.username??``,this.archiveAccount=n,this.verificationEmail&&this.verificationEmail!==n.email){this.progressMessage=``,this.handleAuthError({error:M().GOOGLE_ACCOUNT_MISMATCH,errorCode:`ia_cant_log_user_in`});return}this.moveUserThroughOnboardFlow(e.value)}catch{this.handleAuthError({error:M().GENERIC}),this.endFormTakeover()}}handleAuthError(e){S(`auth handleAuthError`,e);let{error:t=``,cannotUseProvider:n=!1}=e;this.isProcessing=!1,this.errorMessage=t||M().GENERIC,this.rawError={...e},n&&this.resetFormAndLightDom()}startAuthFlow(e){this.provider=e,this.progressMessage=N().GENERIC_WAIT,this.isProcessing=!0,this.errorMessage=``}requestFormTakeover(){this.dispatchEvent(new Event(`formTakeoverRequested`))}endFormTakeover(){this.dispatchEvent(new Event(`formTakeoverEnded`))}get googleAuthButton(){return a`
      <ia-google-auth
        .archiveApiUrl=${this.archiveApiUrl}
        .clientId=${this.googleClientId}
        .csrfToken=${this.csrfToken}
        @onSelected=${e=>{this.sendAnalyticsEvent({action:`google_selected_${e?.detail||`unknown`}`}),this.startAuthFlow(`google`)}}
        @handleAuthError=${e=>{this.sendAnalyticsEvent({action:`google_auth_err_${e?.detail?.error}`}),this.handleAuthError(e.detail)}}
        @profileReceived=${e=>{this.sendAnalyticsEvent({action:`google_auth_profile_received`}),this.startConnectingUserToIA(e)}}
      >
      </ia-google-auth>
    `}static get styles(){return e`
      :host {
        display: block;
        width: 100%;
      }

      :host(:focus) {
        outline: none;
      }

      section {
        width: 100%;
      }

      p {
        font-size: 1.4rem;
        margin-bottom: 2rem;
      }

      .error {
        color: #e51c26;
      }
    `}};y([t({type:Object})],U.prototype,`appServices`,void 0),y([t({type:String})],U.prototype,`archiveApiUrl`,void 0),y([t({type:String})],U.prototype,`googleClientId`,void 0),y([t({type:String})],U.prototype,`verifyUser`,void 0),y([t({type:String})],U.prototype,`verificationEmail`,void 0),y([t({type:String})],U.prototype,`csrfToken`,void 0),y([t({type:Array})],U.prototype,`linkedProviders`,void 0),y([t({type:String})],U.prototype,`referer`,void 0),y([t({type:Boolean})],U.prototype,`embedded`,void 0),y([s()],U.prototype,`fetchHandler`,void 0),y([s()],U.prototype,`analyticsHandler`,void 0),y([s()],U.prototype,`thirdPartyAuthService`,void 0),y([s()],U.prototype,`currentOnboardPhase`,void 0),y([s()],U.prototype,`isProcessing`,void 0),y([s()],U.prototype,`progressMessage`,void 0),y([s()],U.prototype,`errorMessage`,void 0),y([s()],U.prototype,`archiveAccount`,void 0),y([s()],U.prototype,`thirdPartyProfile`,void 0),y([s()],U.prototype,`tokenToClaim`,void 0),y([s()],U.prototype,`provider`,void 0),y([s()],U.prototype,`rawError`,void 0),y([s()],U.prototype,`iaUserToVerify`,void 0),U=y([m(`ia-third-party-auth`)],U);var W=class extends c{constructor(...e){super(...e),this.endpointPath=`/services/account/signup/`,this.inputType=`username`,this.inputRequired=!1,this.csrfToken=``,this.validationStatus=`ready`,this.validationError=``}createRenderRoot(){return this}render(){return a`
      <label for=${this.inputType}>
        <slot name="heading"></slot>
        <slot name="note"></slot>
      </label>
      <div class="input-section">
        <input
          id=${this.inputType}
          name=${this.inputType}
          type=${this.inputType===`username`?`email`:`text`}
          class=${this.validationStatus===`error`?`error`:``}
          minlength=${p(this.minLength)}
          maxlength=${p(this.maxLength)}
          value=${p(this.prefilledValue)}
          data-validity=${this.validationStatus}
          @focus=${this.clearErrors}
          @blur=${this.validate}
          ?required=${this.inputRequired}
        />
        ${l(this.validationStatus!==`ready`,()=>a`<ia-status-indicator
              .mode=${this.validationStatus}
            ></ia-status-indicator>`)}
      </div>
      ${l(this.validationStatus===`error`,()=>this.errorTemplate)}
    `}updated(e){e.has(`suggestedScreenname`)&&this.suggestedScreenname&&this.setupScreennameListener()}get errorTemplate(){return a`<span class="error-msg"
      >${this.validationError?ee(this.sanitizedAndPreppedError):o(`Validation failed`)}</span
    >`}get sanitizedAndPreppedError(){w.addHook(`afterSanitizeAttributes`,e=>{let t=e.getAttribute(`screenname`);t&&(this.suggestedScreenname=t)});let e=w.sanitize(this.validationError,{ALLOWED_TAGS:[`a`],ALLOWED_ATTR:[`screenname`,`href`]});return w.removeHooks(`beforeSanitizeAttributes`),e}setupScreennameListener(){if(!this.suggestedScreenname||this.inputType!==`screenname`)return;let e=this.querySelector(`a[screenname]`);!e||e.getAttribute(`hasListener`)===`true`||(e.addEventListener(`click`,()=>this.applySuggestedScreenname()),e.setAttribute(`hasListener`,`true`))}applySuggestedScreenname(){this.suggestedScreenname&&(this.input.value=this.suggestedScreenname),this.validate()}clearErrors(){this.validationStatus=`ready`,this.validationError=``}async validate(){if(this.validationStatus=`ready`,this.validationError=this.checkForValidationError(),this.validationError){this.validationStatus=`error`;return}this.validationStatus=`loading`;try{let e=await this.checkAvailability(this.input.value);if(!e||!e.success){e?.error&&(this.validationError=e?.error),this.validationStatus=`error`;return}this.validationStatus=`success`}catch{this.validationStatus=`error`}}checkForValidationError(){let e=this.input.value.trim();return this.inputRequired&&!e?o(`Please fill out this field`):this.minLength&&e.length<this.minLength?o(r`Your entry needs to be at least ${this.minLength} characters long`):this.inputType==`screenname`&&/\s{2,}/.test(e)?o(`The screen name cannot have two or more consecutive spaces.`):this.inputType===`screenname`&&/^[._-]|[._-]$/.test(e)?o(`The screen name cannot start or end with a period, dash, or underscore.`):this.input.checkValidity&&!this.input.checkValidity()?o(r`That does not appear to be a valid ${this.inputType===`username`?`email`:`screen name`}`):``}checkAvailability(e){return this.fetchHandler?.fetchApiPathResponse(this.endpointPath,{includeCredentials:!0,includeCsrfToken:!0,method:`POST`,body:JSON.stringify({field:this.inputType,value:e,op:`check`,t:this.csrfToken}),headers:{"Content-Type":`application/json`},retryConfig:te.noRetry})}};y([t({type:Object})],W.prototype,`fetchHandler`,void 0),y([t({type:String})],W.prototype,`endpointPath`,void 0),y([t({type:String})],W.prototype,`inputType`,void 0),y([t({type:String})],W.prototype,`prefilledValue`,void 0),y([t({type:Number})],W.prototype,`minLength`,void 0),y([t({type:Number})],W.prototype,`maxLength`,void 0),y([t({type:Boolean})],W.prototype,`inputRequired`,void 0),y([t({type:String})],W.prototype,`csrfToken`,void 0),y([s()],W.prototype,`validationStatus`,void 0),y([s()],W.prototype,`validationError`,void 0),y([s()],W.prototype,`suggestedScreenname`,void 0),y([h(`input`)],W.prototype,`input`,void 0),W=y([m(`ia-signup-input`),i()],W);var G=class extends c{constructor(...e){super(...e),this.endpointPath=`/services/account/signup/`,this.submissionInProgress=!1,this.usePassword=!0,this.csrfToken=``,this.mailingListPreferences={ml_best_of:!1,ml_events:!1}}render(){return a`
      <form @submit=${this.handleSubmit}>
        ${this.usernameInputTemplate}${this.screennameInputTemplate}
        ${l(this.usePassword,()=>this.passwordInputTemplate)}
        ${this.mailingListsTemplate}
        <div class="signup-error"><slot name="signup-error"></slot></div>
        <ia-button
          mode="primary"
          class="submit-btn"
          type="submit"
          ?loading=${this.submissionInProgress}
        >
          <slot name="button-text">${o(`Sign up`)}</slot>
        </ia-button>
        <span class="terms">
          ${o(a`By signing up, you agree to the Internet Archive's
              <br />
              <a href="/about/terms">Terms of Service</a>`)}
        </span>
      </form>
    `}get usernameInputTemplate(){return a`
      <ia-signup-input
        .inputType=${`username`}
        .maxLength=${127}
        .prefilledValue=${this.prefilledUsername}
        .csrfToken=${this.csrfToken}
        .fetchHandler=${this.fetchHandler}
        .endpointPath=${this.endpointPath}
        ?inputRequired=${!0}
      >
        <span slot="heading" class="heading"
          >${o(`Enter your email address`)}</span
        >
        <span slot="note"> ${o(`Up to 127 characters`)}</span>
      </ia-signup-input>
    `}get screennameInputTemplate(){return a`
      <ia-signup-input
        .inputType=${`screenname`}
        .minLength=${3}
        .maxLength=${40}
        .csrfToken=${this.csrfToken}
        .fetchHandler=${this.fetchHandler}
        .endpointPath=${this.endpointPath}
        ?inputRequired=${!0}
      >
        <span slot="heading" class="heading"
          >${o(`Choose a screen name`)}</span
        >
        <span slot="note"> ${o(`3 to 40 characters`)}</span>
      </ia-signup-input>
    `}get passwordInputTemplate(){return a`
      <div class="form-section">
        <label class="input-label" for="password-input">
          <span class="heading">${o(`Choose a password`)}</span>
          <span class="note"> ${o(`8 to 100 characters`)} </span>
        </label>
        <div class="input-section">
          <input
            type="password"
            name="password"
            id="password-input"
            autocomplete="current-password"
            minlength="8"
            maxlength="100"
            required
          />
          <ia-password-toggle
            .passwordInput=${this.passwordInput}
          ></ia-password-toggle>
        </div>
      </div>
    `}get mailingListsTemplate(){return a`
      <div class="form-section">
        <span class="heading">${o(`Choose announcements`)}</span>
        <mailing-lists
          .mailingLists=${F}
          @selectedMailingList=${this.updateMailingLists}
        >
        </mailing-lists>
      </div>
    `}handleSubmit(e){if(e.preventDefault(),this.formHasErrors)return;let t=new FormData(e.target),n=Object.keys(this.mailingListPreferences).filter(e=>this.mailingListPreferences[e]===!0);t.append(`notifications`,n.join(`,`)),t.append(`t`,this.csrfToken),this.dispatchEvent(new CustomEvent(`formSubmitted`,{detail:{data:Object.fromEntries(t)}}))}updateMailingLists(e){let t=e.detail?.listName;if(!t||this.mailingListPreferences[t]===void 0)return;let n=this.mailingListPreferences;n[e.detail?.listName]=e.detail?.checked,this.mailingListPreferences=n}get formHasErrors(){return!!Array.from(this.formInputs).filter(e=>e.dataset.validity===`loading`||e.dataset.validity===`error`).length}static get styles(){return e`
      form {
        font-size: 1.4rem;
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        gap: 1rem;
        max-width: 570px;
      }

      a {
        color: var(--ia-theme-link-color, #4b64ff);
        text-decoration: none;
      }

      a:hover {
        cursor: pointer;
        text-decoration: underline;
      }

      .heading {
        font-weight: bold;
        font-size: 1.6rem;
        margin-right: 0.3rem;
      }

      .form-section {
        width: 100%;
      }

      ia-signup-input {
        width: 100%;
      }

      .input-section {
        width: 100%;
        display: flex;
        flex-direction: row;
        gap: 1rem;
        align-items: center;
      }

      input.error {
        border-color: var(--ia-theme-error, #e51c23);
      }

      .error-msg,
      .signup-error {
        color: var(--ia-theme-error, #e51c23);
      }

      .signup-error {
        text-align: center;
        align-self: center;
      }

      ia-status-indicator {
        flex: none;
        display: inline-block;
      }

      mailing-lists {
        width: 100%;
      }

      input {
        font-family: inherit;
        font-size: 1.6rem;
        color: inherit;
        width: calc(100% - 2 * 0.5rem);
        flex: none;
        padding: 0.5rem;
        margin: 0.5rem 0;
      }

      .submit-btn {
        align-self: center;
        margin: 0.5rem 0;
      }

      .terms {
        align-self: center;
        text-align: center;
      }
    `}};y([t({type:Object})],G.prototype,`fetchHandler`,void 0),y([t({type:String})],G.prototype,`endpointPath`,void 0),y([t({type:Boolean})],G.prototype,`submissionInProgress`,void 0),y([t({type:Boolean})],G.prototype,`usePassword`,void 0),y([t({type:String})],G.prototype,`prefilledUsername`,void 0),y([t({type:String})],G.prototype,`csrfToken`,void 0),y([s()],G.prototype,`mailingListPreferences`,void 0),y([h(`input[name="password"]`)],G.prototype,`passwordInput`,void 0),y([u(`input`)],G.prototype,`formInputs`,void 0),G=y([m(`ia-signup-form`),i()],G);var K={CodeSubmitted:`codeSubmitted`},q=/^[0-9]+$/,J=/^[a-zA-Z0-9]+$/,Y=class extends c{constructor(){super(...arguments),this.disabled=!1,this.numChars=6,this.numericOnly=!0,this.allowedChars=q}render(){return a`
      ${[...Array(this.numChars).keys()].map(e=>a`<input
            id="OTP-input-${e}"
            part="input"
            type="text"
            autocomplete=${e===0?`one-time-code`:`off`}
            inputmode=${this.numericOnly?`numeric`:`text`}
            ?disabled=${this.disabled}
            @beforeinput=${this.handleInput}
            @paste=${this.handlePaste}
            @keydown=${this.handleKeydown}
          />`)}
    `}firstUpdated(){this.inputs[0].focus()}willUpdate(e){e.has(`numericOnly`)&&(this.allowedChars=this.numericOnly?q:J),e.has(`prefillValue`)&&this.prefillValue!==void 0&&(this.fillInputs(this.prefillValue),this.prefillValue=void 0)}handleInput(e){e.preventDefault();let t=e.target,n=e.data;if(!n)return;if(n.length>1){this.fillInputs(n);return}if(!this.allowedChars.test(n))return;t.value=n;let r=t.nextElementSibling;r&&r.focus(),this.submitIfInputsFilled()}handleKeydown(e){let t=e.target,n=e.key,r=t.previousElementSibling,i=t.nextElementSibling;switch(n){case`Backspace`:case`Delete`:if(e.preventDefault(),r&&r.focus(),t.value===``){r.value=``;break}t.value=``;break;case`Tab`:t.select();break;case`ArrowRight`:case`Right`:e.preventDefault(),i&&i.focus();break;case`ArrowLeft`:case`Left`:e.preventDefault(),r&&r.focus();break}}handlePaste(e){e.preventDefault();let t=e.clipboardData?.getData(`text`);t&&this.fillInputs(t)}fillInputs(e){e===``&&this.clearInputs();let t=e.split(``).filter(e=>this.allowedChars.test(e)).slice(0,this.numChars);if(!(!t||t.length===0)){if(t.forEach((e,t)=>this.inputs[t].value=e),t.length===this.numChars){this.triggerSubmit(t.join(``)),this.inputs[this.numChars-1].focus();return}this.inputs[t.length].focus()}}clearInputs(){this.inputs.forEach(e=>e.value=``),this.inputs[0].focus()}submitIfInputsFilled(){let e=[];this.inputs.forEach(t=>{t.value&&e.push(t.value)}),e.length===this.numChars&&this.triggerSubmit(e.join(``))}triggerSubmit(e){this.dispatchEvent(new CustomEvent(K.CodeSubmitted,{detail:this.numericOnly?e:e.toUpperCase(),bubbles:!0,composed:!0}))}static get styles(){return[v,e`
        :host {
          --primary-text-color--: var(--primary-text-color);
          --font-size-lg--: var(--font-size-lg);
        }

        :host {
          display: flex;
          flex-direction: row;
          flex-wrap: nowrap;
          gap: 5px;
        }

        input {
          color: var(--primary-text-color--);
          font-size: var(--font-size-lg--);
          width: var(--font-size-lg--);
          font-weight: bold;
          height: calc(var(--font-size-lg--) + 1.25rem);
          text-align: center;
          text-transform: uppercase;
          padding: 0;
        }
      `]}};g([t({type:String})],Y.prototype,`prefillValue`,void 0),g([t({type:Boolean})],Y.prototype,`disabled`,void 0),g([t({type:Number})],Y.prototype,`numChars`,void 0),g([t({type:Boolean})],Y.prototype,`numericOnly`,void 0),g([t({type:Object})],Y.prototype,`allowedChars`,void 0),g([u(`input`)],Y.prototype,`inputs`,void 0),Y=g([_(`ia-otp-input`)],Y);var ne={NewCodeRequested:`newCodeRequested`,CodeSubmitted:`codeSubmitted`},X=class extends c{constructor(){super(...arguments),this.validationStatus=`ready`,this.newCodeSending=!1,this.numPasscodeChars=6,this.numericOnly=!0}render(){return a`
      <div class="input-section">
        <ia-otp-input
          .numChars=${this.numPasscodeChars}
          ?numericOnly=${this.numericOnly}
          ?disabled=${this.validationStatus===`loading`||this.validationStatus===`success`}
        ></ia-otp-input>
        <ia-status-indicator
          part="status-indicator"
          .mode=${this.validationStatus}
        ></ia-status-indicator>
      </div>
      ${this.validationStatus===`error`?a`<p class="error-msg">
            ${o(`The code entered is invalid or expired`)}
          </p>`:n}
      ${this.resendCodeButtonTemplate}
    `}willUpdate(e){e.has(`validationStatus`)&&this.OTPInput&&this.validationStatus===`error`&&(this.OTPInput.prefillValue=``),e.has(`newCodeSending`)&&this.newCodeSending&&this.OTPInput&&(this.OTPInput.prefillValue=``)}get resendCodeButtonTemplate(){return this.newCodeSending?a`<span part="new-code-message" class="new-code-msg"
          >${o(`Emailing...`)}</span
        >`:a`
          <ia-button
            mode="link"
            class="new-code-btn"
            part="new-code-button"
            .disabled=${this.validationStatus===`loading`||this.validationStatus===`success`}
            @click=${this.handleNewCodeRequested}
          >
            ${o(`Email me another code`)}
          </ia-button>
        `}async handleNewCodeRequested(){this.dispatchEvent(new CustomEvent(ne.NewCodeRequested,{bubbles:!0,composed:!0})),this.OTPInput.prefillValue=``}static get styles(){return[v,e`
        :host {
          --font-size-standard--: var(--font-size-standard);
          --font-size-lg--: var(--font-size-lg);
          --color-success--: var(--color-success);
          --color-danger--: var(--color-danger);
          --link-color--: var(--link-color);

          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
        }

        .input-section {
          display: flex;
          flex-direction: row;
          align-items: center;
          justify-content: center;
          gap: 5px;
        }

        ia-status-indicator {
          --icon-width: calc(var(--font-size-lg--) * 1.33);
        }

        .error-msg {
          margin-top: 10px;
          font-size: var(--font-size-standard--);
          color: var(--color-danger--);
          margin-bottom: -10px;
        }

        .new-code-msg {
          margin-top: 10px;
          font-size: var(--font-size-standard--);
          color: var(--link-color--);
          display: flex;
          flex-direction: column;
          justify-content: center;
        }
      `]}};g([t({type:String})],X.prototype,`validationStatus`,void 0),g([t({type:Boolean})],X.prototype,`newCodeSending`,void 0),g([t({type:Number})],X.prototype,`numPasscodeChars`,void 0),g([t({type:Boolean})],X.prototype,`numericOnly`,void 0),g([h(`ia-otp-input`)],X.prototype,`OTPInput`,void 0),X=g([_(`ia-otp-form`),i()],X);var Z=class extends c{constructor(...e){super(...e),this.promptState=`form`,this.validationStatus=`ready`,this.newCodeSending=!1,this.PASSCODE_TTL_MINUTES=10,this.RATE_LIMIT_MINUTES=10}render(){return f(this.promptState,[[`form`,()=>this.formTemplate],[`log-in`,()=>this.logInTemplate],[`sign-up`,()=>this.signupTemplate],[`cannot-email-code`,()=>this.cannotEmailTemplate],[`too-many-attempts`,()=>this.tooManyAttemptsTemplate]])??n}get formTemplate(){return a`
      <form class="otp-content">
        <p class="content-header">
          ${o(`A 6-digit code has been emailed to:`)} <br />${this.email??o(`your email address`)}
        </p>
        <p class="content-body">
          ${o(r`Please enter your code below within the next ${this.PASSCODE_TTL_MINUTES} minutes`)}
        </p>
        <ia-otp-form
          .validationStatus=${this.validationStatus}
          ?newCodeSending=${this.newCodeSending}
        ></ia-otp-form>
      </form>
    `}get logInTemplate(){return a`
      <div class="otp-content">
        <p class="content-header align-left">
          ${o(`Enter your email address`)}
        </p>
        <form class="login-form" @submit=${this.handleLoginFormSubmitted}>
          <input
            id="email"
            type="email"
            class="email-input"
            value=${this.email??``}
          />
          <ia-button
            mode="primary"
            class="email-code-btn"
            type="submit"
            .loadingText=${o(`Emailing...`)}
            ?loading=${this.newCodeSending}
          >
            ${o(`Email me a code`)}
          </ia-button>
        </form>
      </div>
    `}get signupTemplate(){return a`
      <ia-signup-form
        .fetchHandler=${this.fetchHandler}
        .csrfToken=${this.csrfToken??``}
        .usePassword=${!1}
        ?submissionInProgress=${this.newCodeSending}
        @formSubmitted=${this.handleSignupFormSubmitted}
      >
        <span slot="button-text">${o(`Email me a code`)}</span>
      </ia-signup-form>
    `}get cannotEmailTemplate(){return a`
      <div class="otp-content">
        <p class="content-header">
          ${o(a`A code cannot be emailed to <br />
              ${this.email??`your email address`} at this time`)}
        </p>
        <p class="content-body">
          ${o(r`Please try again in ${this.RATE_LIMIT_MINUTES} minutes.`)}
        </p>
        <ia-button
          class="close-btn"
          mode="primary"
          @click=${this.requestModalClose}
          >${o(`Okay`)}</ia-button
        >
      </div>
    `}get tooManyAttemptsTemplate(){return a`
      <div class="otp-content">
        <p class="content-header">
          ${o(a`There have been too many failed attempts <br />
              for ${this.email??`your email address`}`)}
        </p>
        <p class="content-body">
          ${o(r`You may wait ${this.RATE_LIMIT_MINUTES} minutes and restart the process.`)}
        </p>
        <ia-button
          class="close-btn"
          mode="primary"
          @click=${this.requestModalClose}
        >
          ${o(`Okay`)}
        </ia-button>
      </div>
    `}async handleLoginFormSubmitted(e){e.preventDefault();let t=this.shadowRoot?.querySelector(`form`);t&&t.reportValidity()&&(this.email=t.email.value.trim(),this.dispatchEvent(new CustomEvent(`newCodeRequested`,{detail:{email:this.email,showBackButton:!0}})))}handleSignupFormSubmitted(e){this.email=e.detail.data.username.trim(),this.email&&this.dispatchEvent(new CustomEvent(`newCodeRequested`,{detail:{email:this.email,screenname:e.detail.data.screenname?.trim(),notifications:e.detail.data.notifications?.trim(),showBackButton:!0}}))}requestModalClose(){this.dispatchEvent(new CustomEvent(`modalCloseRequested`))}static get styles(){return e`
      .otp-content {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: 0 10px;
      }

      p {
        text-align: center;
        color: var(--ia-theme-primary-text-color, #2c2c2c);
      }

      .content-header {
        font-size: 1.6rem;
        font-weight: bold;
        margin: 0;
      }

      .align-left {
        align-self: flex-start;
      }

      .login-form {
        width: 100%;
        display: flex;
        align-items: center;
        flex-direction: column;
      }

      ia-signup-form {
        display: block;
        width: calc(100% - 35px);
      }

      .email-input {
        align-self: flex-start;
        width: calc(100% - 2 * 0.5rem);
        color: inherit;
        font-family: inherit;
        padding: 0.5rem;
        margin: 1rem 0;
      }

      .content-body {
        font-size: 1.4rem;
        margin: 10px;
      }
    `}};y([t({type:String})],Z.prototype,`email`,void 0),y([t({type:String})],Z.prototype,`promptState`,void 0),y([t({type:Object})],Z.prototype,`appServices`,void 0),y([t({type:Object})],Z.prototype,`fetchHandler`,void 0),y([t({type:String})],Z.prototype,`csrfToken`,void 0),y([t({type:String})],Z.prototype,`validationStatus`,void 0),y([s()],Z.prototype,`newCodeSending`,void 0),Z=y([m(`ia-otp-prompt`),i()],Z);var Q=class extends c{constructor(...e){super(...e),this.endpointPath=`/services/account/otp/`,this.csrfToken=``,this.buttonStyle=`button`,this.promptState=`form`,this.validationStatus=`ready`,this.newCodeSending=!1,this.showBackButton=!1,this.DEFAULT_HEADER_COLOR=`#194880`,this.ERROR_HEADER_COLOR=`#720D11`}render(){return a`
      <ia-button
        class="request-code-btn"
        mode=${this.buttonStyle===`link`?`link`:`custom`}
        @click=${this.handleOTPButtonPressed}
      >
        <div class="button-content">
          ${l(this.buttonStyle!==`link`,()=>this.emailIconTemplate)}
          <span class="button-text">
            ${this.newCodeSending?o(`Emailing...`):a`<slot></slot>`}
          </span>
        </div>
      </ia-button>
    `}willUpdate(e){e.has(`appServices`)&&this.appServices&&this.setupAppServices(),this.modalContent&&(e.has(`newCodeSending`)&&(this.modalContent.newCodeSending=this.newCodeSending),e.has(`validationStatus`)&&(this.modalContent.validationStatus=this.validationStatus),e.has(`promptState`)&&(this.modalContent.promptState=this.promptState,this.showModal()))}get emailIconTemplate(){return a`<ia-icon .src=${E}></ia-icon>`}async setupAppServices(){let e=await this.appServices?.get();[this.modalManager,this.fetchHandler]=await Promise.all([e?.modalManager.get(),e?.fetchHandler.get()])}handleOTPButtonPressed(){if(this.hostPageName===`log in`)return this.showLoginForm();if(this.hostPageName===`sign up`)return this.showSignupForm();this.requestNewCode()}handleNewCodeRequested(e){e.detail?.email&&(this.email=e.detail.email),e.detail?.screenname&&(this.requestedScreenname=e.detail.screenname),e.detail?.notifications&&(this.requestedNotifications=e.detail.notifications),e.detail?.showBackButton&&(this.showBackButton=e.detail.showBackButton),this.requestNewCode()}async requestNewCode(){if(!this.email)return this.showLoginForm();this.newCodeSending=!0,this.validationStatus=`ready`;try{let e={email:this.email,token:this.csrfToken};if(this.hostPageName&&(e.sender_page=this.hostPageName),!this.fetchHandler){this.promptState=`cannot-email-code`;return}let t=await this.fetchHandler.fetchApiPathResponse(this.endpointPath,{method:`POST`,body:JSON.stringify(e),includeCredentials:!0,includeCsrfToken:!0,headers:{"Content-Type":`application/json`}});this.promptState=t?.success?`form`:`cannot-email-code`}catch(e){console.error(e),this.promptState=`cannot-email-code`}finally{this.newCodeSending=!1,this.showModal()}}async validateCode(e){if(!(!this.email||!e)){this.validationStatus=`loading`;try{let t={email:this.email,passcode:e,token:this.csrfToken};if(this.hostPageName&&(t.sender_page=this.hostPageName),this.requestedScreenname&&(t.screenname=this.requestedScreenname),this.requestedNotifications&&(t.notifications=this.requestedNotifications),!this.fetchHandler){this.validationStatus=`error`;return}let n=await this.fetchHandler.fetchApiPathResponse(this.endpointPath,{method:`POST`,body:JSON.stringify(t),includeCredentials:!0,includeCsrfToken:!0,headers:{"Content-Type":`application/json`}});if(n?.success===!0){this.validationStatus=`success`,setTimeout(()=>{this.closeModal(),this.dispatchEvent(new CustomEvent(`otpValidated`,{detail:{token:n.value??``,email:this.email}}))},1e3);return}n?.error===`rate_exception`&&(this.promptState=`too-many-attempts`),this.validationStatus=`error`}catch(e){console.error(e),this.validationStatus=`error`}}}showModal(){this.modalManager?.classList.add(`default`);let e=new b({title:this.modalTitle,headerColor:this.modalHeaderColor,showCloseButton:!0,showLeftNavButton:this.showBackButton,leftNavButtonText:`Back`,showHeaderLogo:!1,closeOnBackdropClick:!1});this.modalManager?.showModal({config:e,userClosedModalCallback:()=>this.closeModal(),userPressedLeftNavButtonCallback:()=>this.handleBackButtonPressed(),customModalContent:a`
        <ia-otp-prompt
          .email=${this.email}
          .promptState=${this.promptState}
          .validationStatus=${this.validationStatus}
          .newCodeSending=${this.newCodeSending}
          .fetchHandler=${this.fetchHandler}
          .csrfToken=${this.csrfToken}
          @newCodeRequested=${e=>this.handleNewCodeRequested(e)}
          @codeSubmitted=${e=>this.validateCode(e.detail)}
          @modalCloseRequested=${()=>this.closeModal()}
        ></ia-otp-prompt>
      `})}closeModal(){this.modalManager?.closeModal(),this.modalManager?.classList.remove(`default`)}handleBackButtonPressed(){switch(this.hostPageName){case`log in`:case`sign in`:this.showLoginForm();break;case`sign up`:this.showSignupForm();break}}showLoginForm(){this.promptState=`log-in`,this.showBackButton=!1,this.showModal()}showSignupForm(){this.promptState=`sign-up`,this.showBackButton=!1,this.showModal()}get modalContent(){return this.modalManager?.shadowRoot?.querySelector(`ia-otp-prompt`)}get modalTitle(){return a`${f(this.promptState,[[`form`,()=>o(`Enter code`)],[`log-in`,()=>o(`Log in`)],[`sign-up`,()=>o(`Sign up`)],[`cannot-email-code`,()=>o(`Cannot email code`)],[`too-many-attempts`,()=>o(`Too many attempts`)]])}`}get modalHeaderColor(){return this.promptState===`too-many-attempts`||this.promptState===`cannot-email-code`?this.ERROR_HEADER_COLOR:this.DEFAULT_HEADER_COLOR}static get styles(){return e`
      ia-button {
        --ia-button-custom-text-color: var(--ia-theme-primary-text-color, #333);
        --ia-button-custom-border: var(--third-party-border-color, #dadce0);
        --ia-button-custom-fill: #fff;
        --ia-button-custom-active-fill: var(
          --third-party-bg-color,
          rgba(66, 133, 244, 0.08)
        );
        --ia-button-custom-active-border: var(
          --third-party-border-color,
          rgb(210, 227, 252)
        );
        --ia-button-transition: background-color 0.2s;
        --ia-theme-button-padding: 0;
        --ia-theme-button-height: fit-content;
      }

      .button-content {
        font-size: 1.4rem;
        min-width: 224px;
        height: 40px;
        padding: 0 12px;
        display: flex;
        flex-direction: row;
        gap: 15px;
        justify-content: space-between;
        align-items: center;
      }

      .button-text {
        flex-grow: 1;
      }
    `}};y([t({type:Object})],Q.prototype,`appServices`,void 0),y([t({type:String})],Q.prototype,`endpointPath`,void 0),y([t({type:String})],Q.prototype,`email`,void 0),y([t({type:String})],Q.prototype,`csrfToken`,void 0),y([t({type:String})],Q.prototype,`hostPageName`,void 0),y([t({type:String})],Q.prototype,`buttonStyle`,void 0),y([s()],Q.prototype,`promptState`,void 0),y([s()],Q.prototype,`validationStatus`,void 0),y([s()],Q.prototype,`newCodeSending`,void 0),y([s()],Q.prototype,`showBackButton`,void 0),y([s()],Q.prototype,`modalManager`,void 0),y([s()],Q.prototype,`fetchHandler`,void 0),y([s()],Q.prototype,`requestedScreenname`,void 0),y([s()],Q.prototype,`requestedNotifications`,void 0),Q=y([m(`ia-otp-modal`),i()],Q);var $=class extends c{constructor(...e){super(...e),this.csrfToken=``,this.useThirdPartyTakeover=!1,this.showThirdPartyOption=!0,this.googleClientId=``,this.archiveApiUrl=`archive.org`,this.dividerPosition=`bottom`}render(){return a`
      <div class="auth-options">
        ${l(this.googleClientId&&this.showThirdPartyOption,()=>this.thirdPartyAuthTemplate)}
        ${l(!this.useThirdPartyTakeover&&this.dividerPosition===`middle`,()=>a`<ia-text-divider class="middle"></ia-text-divider>`)}
        ${l(!this.useThirdPartyTakeover,()=>this.OTPTemplate)}
        ${l(!this.useThirdPartyTakeover&&this.dividerPosition===`bottom`,()=>a`<ia-text-divider class="bottom"></ia-text-divider>`)}
      </div>
    `}get thirdPartyAuthTemplate(){return a`
      <div slot="ia-third-party-auth" class="third-party-slot">
        <ia-third-party-auth
          .appServices=${this.appServices}
          .archiveApiUrl=${this.archiveApiUrl}
          .csrfToken=${this.csrfToken}
          .googleClientId=${this.googleClientId}
          .referer=${this.referer}
          .verifyUser=${this.userToVerify?.identifier??``}
          .verificationEmail=${this.userToVerify?.email??``}
          @formTakeoverRequested=${()=>this.dispatchEvent(new Event(`formTakeoverRequested`))}
          @formTakeoverEnded=${()=>this.dispatchEvent(new Event(`formTakeoverEnded`))}
          @IAThirdPartyAuthVerified=${e=>this.dispatchEvent(new CustomEvent(`IAThirdPartyAuthVerified`,{detail:{email:e.detail.email}}))}
          @IAThirdPartyAuth:MGCToken=${e=>this.dispatchEvent(new CustomEvent(`IAThirdPartyAuth:MGCToken`,{detail:{token:e.detail.token}}))}
        ></ia-third-party-auth>
      </div>
    `}get OTPTemplate(){return a`
      <ia-otp-modal
        .email=${this.userToVerify?.email}
        .appServices=${this.appServices}
        .hostPageName=${this.hostPageName}
        .csrfToken=${this.csrfToken??``}
        @otpValidated=${e=>this.dispatchEvent(new CustomEvent(`otpValidated`,{detail:{token:e.detail.token??``,email:e.detail.email??``}}))}
      >
        <slot name="otp-message">${o(`Email me a code`)}</slot>
      </ia-otp-modal>
    `}static get styles(){return e`
      :host {
        width: 100%;
      }
      .auth-options {
        width: calc(100% - 40px);
        margin: 10px auto 20px auto;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 0;
      }

      ia-text-divider {
        width: 100%;
      }

      ia-text-divider.bottom {
        margin-top: 20px;
      }

      ia-text-divider.middle {
        margin-bottom: 15px;
      }
    `}};y([t({type:Object})],$.prototype,`appServices`,void 0),y([t({type:Object})],$.prototype,`userToVerify`,void 0),y([t({type:String})],$.prototype,`csrfToken`,void 0),y([t({type:String})],$.prototype,`referer`,void 0),y([t({type:Boolean})],$.prototype,`useThirdPartyTakeover`,void 0),y([t({type:Boolean})],$.prototype,`showThirdPartyOption`,void 0),y([t({type:String})],$.prototype,`googleClientId`,void 0),y([t({type:String})],$.prototype,`archiveApiUrl`,void 0),y([t({type:String})],$.prototype,`hostPageName`,void 0),y([t({type:String})],$.prototype,`dividerPosition`,void 0),$=y([m(`ia-auth-options`),i()],$);