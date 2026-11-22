import{n as e}from"./rolldown-runtime-QTnfLwEv.js";import{A as t,C as n,E as r,N as i,O as a,P as o,S as s,T as c,_ as l,l as u,u as d,v as f,w as p,x as m}from"./vendor-lit-MQ4DzmMc.js";import{a as h,o as g,t as _}from"./index-NFRVgqV2.js";import{i as v}from"./esm-CWeBglMZ.js";import{t as y}from"./purify.es-RmFZhtLI.js";import"./ia-button-DYuVUpnO.js";import"./ia-auth-options-DIzxHnZF.js";import{t as b}from"./ia-logo-gray-circle-BADeBCkf.js";var x=class extends c{constructor(...e){super(...e),this.endpointPath=`/services/account/login/`,this.pkceStatus=`ready`,this.hideIAForm=!1}render(){return a`<div id="page-container">
      <section class="form-wrapper">
        ${l(!this.hideIAForm,()=>a`
            <img src=${b} alt=${o(`Internet Archive Logo`)} />
            <h1>${o(`Sign In to the Internet Archive`)}</h1>
          `)}
        ${this.loginOptionsTemplate}
        ${l(this.pkceStatus===`loading`,()=>o(`Authenticating...`))}
        ${l(this.pkceStatus===`failed`,()=>this.pkceFailedTemplate)}
      </section>
    </div>`}willUpdate(e){e.has(`appServices`)&&this.appServices&&this.checkIfLoggedIn()}get loginOptionsTemplate(){return a`
      <ia-auth-options
        .appServices=${this.appServices}
        .csrfToken=${this.csrfToken}
        .googleClientId=${this.googleClientId}
        .archiveApiUrl=${this.archiveApiUrl}
        .hostPageName=${`sign in`}
        .dividerPosition=${`middle`}
        ?useThirdPartyTakeover=${this.hideIAForm}
        @formTakeoverRequested=${()=>this.hideIAForm=!0}
        @formTakeoverEnded=${()=>this.hideIAForm=!1}
        @IAThirdPartyAuthVerified=${e=>{this.handleLoginSuccess(e.detail.email)}}
        @otpValidated=${e=>{this.username=e.detail.email,this.handleLoginSuccess(e.detail.email)}}
      >
        <span slot="otp-message">${o(`Email me a code to sign in`)}</span>
      </ia-auth-options>
    `}get pkceFailedTemplate(){return a`
      <span class="error">
        ${o(`Authorization error: You can't log into this app because it isn't configured correctly. Please contact the app's developer if the problem persists. (Error: missing redirect_uri)`)}
      </span>
    `}async checkIfLoggedIn(){await(await this.appServices?.get())?.userService.get().then(e=>e.getLoggedInUser()).then(e=>{e.success&&this.handleLoginSuccess(e.success.username)})}handleLoginSuccess(e){e&&(this.username=e),this.handlePostAuthPKCEFlow()}async handlePostAuthPKCEFlow(){if(!this.pkceParams||!this.pkceParams.redirect_uri)return this.handlePKCEFailed();this.pkceStatus=`loading`;let e;try{e=new URL(this.pkceParams.redirect_uri)}catch{return this.handlePKCEFailed()}if(!e)return this.handlePKCEFailed();let t=await this.addPKCEParams(e);if(!t)return this.handlePKCEFailed();this.redirectTo(t)}handlePKCEFailed(){this.pkceStatus=`failed`}redirectTo(e){window.location.href=e}async addPKCEParams(e){if(!this.pkceParams?.state)return e.searchParams.set(`error`,`invalid_request`),e.toString();if(e.searchParams.append(`state`,this.pkceParams.state),this.pkceParams.response_type!==`code`)return e.searchParams.set(`error`,`unsupported_response_type`),e.toString();try{let t=await this.requestPKCECode();return t?!t.success||!t?.value?.code?(e.searchParams.set(`error`,`access_denied`),e.toString()):(e.searchParams.append(`code`,t.value.code),e.toString()):(e.searchParams.set(`error`,`server_error`),e.toString())}catch{return e.searchParams.set(`error`,`server_error`),e.toString()}}async requestPKCECode(){if(this.pkceParams)return this.fetchHandler?.fetchApiPathResponse(this.endpointPath,{includeCredentials:!0,includeCsrfToken:!0,method:`POST`,body:JSON.stringify({username:this.username,code_challenge:this.pkceParams.code_challenge,client_id:this.pkceParams.client_id,redirect_uri:this.pkceParams.redirect_uri,code_challenge_method:this.pkceParams.code_challenge_method,t:this.csrfToken}),headers:{"Content-Type":`application/json`},retryConfig:h.noRetry})}static get styles(){return t`
      #page-container {
        background-color: var(--ia-theme-secondary-background-color, white);
        min-height: 100vh;
        font-size: 1.4rem;
      }

      .form-wrapper {
        margin: 0 auto;
        max-width: 60rem;
        display: flex;
        flex-direction: column;
        align-items: center;
      }

      h1 {
        font-size: 2.4rem;
        font-weight: 500;
        margin: 0;
        line-height: 1.1;
        text-align: center;
      }

      img {
        height: 8rem;
        margin: 3rem 0;
      }

      ia-auth-options {
        margin-top: 20px;
      }

      .status-message {
        margin: 0;
      }

      .error {
        color: #cc0000;
        margin-top: 1rem;
      }

      @media only screen and (max-width: 992px) {
        .form-wrapper {
          max-width: 45rem;
          width: 100%;
        }
      }

      @media only screen and (max-width: 768px) {
        .form-wrapper {
          max-width: 43rem;
          width: 100%;
        }
      }

      @media only screen and (max-width: 480px) {
        .form-wrapper {
          width: 30rem;
          padding: 0 20px;
        }
      }
    `}};_([n({type:Object})],x.prototype,`appServices`,void 0),_([n({type:String})],x.prototype,`endpointPath`,void 0),_([n({type:String})],x.prototype,`prefilledUsername`,void 0),_([n({type:Object})],x.prototype,`pkceParams`,void 0),_([n({type:String})],x.prototype,`csrfToken`,void 0),_([n({type:Object})],x.prototype,`fetchHandler`,void 0),_([n({type:String})],x.prototype,`googleClientId`,void 0),_([n({type:String})],x.prototype,`archiveApiUrl`,void 0),_([s()],x.prototype,`pkceStatus`,void 0),_([s()],x.prototype,`hideIAForm`,void 0),_([s()],x.prototype,`username`,void 0),x=_([p(`ia-pkce-sign-in`),i()],x);var S=e({default:()=>w}),C=class extends c{constructor(...e){super(...e),this.endpointPath=`/services/account/login/`,this.loginStatus=`ready`,this.submissionInProgress=!1,this.hideIAForm=!1,this.emailVerificationEndpoint=`/services/account/verify-email/`}render(){return a`<div id="page-container">
      <section class="form-wrapper">
        ${this.pkceParams?this.pkceSignInTemplate:this.formTemplate}
      </section>
    </div>`}willUpdate(e){e.has(`appServices`)&&this.appServices&&this.setupAppServices(),e.has(`referer`)&&this.referer&&this.validateReferer()}get pkceSignInTemplate(){return a`
      <ia-pkce-sign-in
        .appServices=${this.appServices}
        .endpointPath=${this.endpointPath}
        .prefilledUsername=${this.prefilledUsername}
        .pkceParams=${this.pkceParams}
        .csrfToken=${this.csrfToken}
        .fetchHandler=${this.fetchHandler}
        .googleClientId=${this.googleClientId}
        .archiveApiUrl=${this.archiveApiUrl}
      ></ia-pkce-sign-in>
    `}get formTemplate(){return a`
      ${this.hideIAForm?r:a`<img src=${b} alt=${o(`Internet Archive Logo`)} />
            <h1>${o(`Log In`)}</h1>
            <p>
              ${o(`Don't have an account?`)}
              <a
                class="sign-up-link"
                href="/signup${l(this.referer,()=>`?referer=`+encodeURIComponent(this.referer??``))}"
              >
                ${o(`Sign up for free!`)}
              </a>
            </p>`}
      ${this.loginOptionsTemplate}
      ${l(!this.hideIAForm,()=>this.IALoginTemplate)}
    `}get loginOptionsTemplate(){return a`
      <ia-auth-options
        .appServices=${this.appServices}
        .csrfToken=${this.csrfToken}
        .googleClientId=${this.googleClientId}
        .referer=${this.referer}
        .archiveApiUrl=${this.archiveApiUrl}
        .hostPageName=${`log in`}
        ?useThirdPartyTakeover=${this.hideIAForm}
        @formTakeoverRequested=${()=>this.hideIAForm=!0}
        @formTakeoverEnded=${()=>this.hideIAForm=!1}
        @IAThirdPartyAuthVerified=${()=>this.handleSuccess()}
        @otpValidated=${this.handleSuccess}
      >
        <span slot="otp-message">${o(`Email me a code to log in`)}</span>
      </ia-auth-options>
    `}get IALoginTemplate(){return a`
      <form @submit=${this.handleSubmit}>
        ${this.usernameInputTemplate}${this.passwordInputTemplate}
        ${this.statusMessageTemplate}${this.checkboxTemplate}
        <ia-button
          class="submit-btn"
          mode="primary"
          type="submit"
          ?loading=${this.submissionInProgress}
        >
          ${o(`Log in`)}
        </ia-button>
      </form>
    `}get usernameInputTemplate(){return a`
      <label class="input-label" for="email-input"
        >${o(`Email address`)}</label
      >
      <input
        type="email"
        name="username"
        id="email-input"
        autocomplete="username"
        value=${f(this.prefilledUsername)}
        required
      />
    `}get passwordInputTemplate(){return a`
      <label class="input-label" for="password-input">
        ${o(`Password`)}
      </label>
      <div class="password-input">
        <input
          type="password"
          name="password"
          id="password-input"
          autocomplete="current-password"
          required
        />
        <ia-password-toggle
          .passwordInput=${this.passwordInput}
        ></ia-password-toggle>
      </div>
    `}get checkboxTemplate(){return a`
      <div class="checkbox">
        <input type="checkbox" name="remember" id="remember-input" checked />
        <label for="remember-input">${o(`Keep me logged in`)}</label>
      </div>
    `}get statusMessageTemplate(){return a`
      <p class="status-message">
        ${d(this.loginStatus,[[`ready`,()=>this.forgotPasswordLinkTemplate],[`login_error`,()=>this.loginErrorTemplate],[`login_generic_error`,()=>this.loginFailedTemplate],[`missing_fields`,()=>this.missingFieldsTemplate],[`account_not_verified`,()=>this.accountUnverifiedTemplate],[`account_max_unverified`,()=>this.accountMaxUnverifiedTemplate],[`verification_email_sent`,()=>this.verificationEmailSentTemplate]])}
      </p>
    `}get forgotPasswordLinkTemplate(){return a`<a href="/forgot-password"> ${o(`Forgot password?`)} </a>`}get loginErrorTemplate(){return this.loginError?a`<span class="error">${u(this.sanitize(this.loginError))}</span>`:this.loginFailedTemplate}get missingFieldsTemplate(){return a`
      <span class="error">${o(`Please enter an email and password.`)}</span>
      ${this.forgotPasswordLinkTemplate}
    `}get loginFailedTemplate(){return a`
      <span class="error">
        ${o(`Sorry, we are unable to log you in at this time.`)}
      </span>
    `}get accountUnverifiedTemplate(){return a`
      <span class="error">
        ${o(`Your email address needs to be verified to continue logging in.`)}
        <ia-button mode="link" @click=${this.resendVerificationEmail}>
          ${o(`Resend verification email.`)}
        </ia-button>
      </span>
    `}get accountMaxUnverifiedTemplate(){return a`
      <span class="error">
        ${o(`Too many verification emails have been sent to this address.`)}
      </span>
      <a href="mailto:info@archive.org">
        ${o(`Contact info@archive.org for further assistance.`)}
      </a>
    `}get verificationEmailSentTemplate(){return a`
      ${o(`Verification email has been sent. Please check your inbox or spam folder for the verification email and click on the link inside. Contact info@archive.org for further assistance.`)}
    `}async setupAppServices(){let e=await this.appServices?.get();await Promise.all([e?.pageMetadataService.get().then(e=>{this.pageMetadataService=e,this.setupPageMetadata()}),e?.fetchHandler.get().then(e=>{this.fetchHandler=e}),e?.csrfService.get().then(e=>this.updateCSRFToken(e)),e?.appConfig.get().then(e=>{this.googleClientId=e?.googleClientId,this.archiveApiUrl=e?.archiveApiUrl})])}async updateCSRFToken(e){this.csrfToken=await e.getToken()}async setupPageMetadata(){let e=this.pkceParams?o(`Sign In to the Internet Archive`):o(`Log in to the Internet Archive`);this.pageMetadataService?.setTitle(e),this.pageMetadataService?.addLink({rel:`canonical`,path:`/login`})}validateReferer(){this.referer&&(g(this.referer)||(this.referer=void 0))}async handleSubmit(e){if(e.preventDefault(),!(this.submissionInProgress||!this.validateFormFields())){this.submissionInProgress=!0,this.loginStatus=`ready`,this.loginError&&(this.loginError=``);try{let e=await this.requestLogin();if(e?.success){this.handleSuccess();return}if(this.submissionInProgress=!1,e?.value===`account_not_verified`||e?.value===`account_max_unverified`){this.loginStatus=e.value;return}if(e?.error){this.loginError=e.error,this.loginStatus=`login_error`;return}this.loginStatus=`login_generic_error`}catch{this.submissionInProgress=!1,this.loginStatus=`login_generic_error`}}}validateFormFields(){if(!this.form.reportValidity())return!1;let e=this.form.username.value,t=this.form.password.value;return!e||!t?(this.loginStatus=`missing_fields`,!1):!0}async requestLogin(){return this.fetchHandler?.fetchApiPathResponse(this.endpointPath,{includeCredentials:!0,includeCsrfToken:!0,method:`POST`,body:JSON.stringify({username:this.form.username.value,password:this.form.password.value,remember:this.form.remember.checked.toString(),t:this.csrfToken}),headers:{"Content-Type":`application/json`},retryConfig:h.noRetry})}async resendVerificationEmail(){try{let e=await this.fetchHandler?.fetchApiPathResponse(this.emailVerificationEndpoint,{includeCredentials:!0,includeCsrfToken:!0,method:`POST`,body:JSON.stringify({username:this.form.username.value}),headers:{"Content-Type":`application/json`},retryConfig:h.noRetry});if(e?.success){this.loginStatus=`verification_email_sent`;return}e?.error?.toLowerCase().includes(`csrf`)&&v(Error(`CSRF token rejected on resend-verification-email request`)),this.loginStatus=`account_max_unverified`}catch{this.loginStatus=`account_max_unverified`}}sanitize(e){return y.sanitize(e,{ALLOWED_TAGS:[`a`],ALLOWED_ATTR:[`href`,`target`]})}handleSuccess(){if(this.successCallback){this.successCallback();return}this.redirectTo(this.referer??`/`)}redirectTo(e){window.location.href=e}static get styles(){return t`
      #page-container {
        background-color: var(--ia-theme-secondary-background-color, white);
        min-height: var(--loginPageContainerMinHeight, 100vh);
        margin-bottom: var(--loginPageContainerMarginBottom, 0);
        font-size: 1.4rem;
      }

      .form-wrapper {
        margin: 0 auto;
        max-width: 60rem;
        display: flex;
        flex-direction: column;
        align-items: center;
      }

      h1 {
        font-size: 2.4rem;
        font-weight: 500;
        margin: 0;
        line-height: 1.1;
        text-align: center;
      }

      img {
        height: 8rem;
        margin: 3rem 0;
      }

      p {
        margin: 10px 0;
      }

      form {
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        align-self: flex-start;
        gap: 5px;
        padding: 0 20px;
        width: 100%;
      }

      .input-label {
        font-size: 1.6rem;
        font-weight: bold;
      }

      input[type='password'],
      input[type='text'],
      input[type='email'] {
        width: calc(100% - 60px);
        padding: 3px;
        font-size: 1.6rem;
        font-family: inherit;
        padding: 5px;
      }

      input[type='email'] {
        margin-bottom: 10px;
      }

      .password-input {
        width: 100%;
        display: flex;
        flex-direction: row;
        gap: 15px;
        align-items: space-between;
      }

      .status-message {
        margin: 0;
      }

      .checkbox {
        margin-top: 20px;
        display: flex;
        flex-direction: row;
        align-items: flex-start;
        gap: 0.5rem;
      }

      .error {
        color: #cc0000;
        margin-top: 1rem;
      }

      a:link,
      a:visited {
        color: var(--ia-theme-link-color);
        text-decoration: none;
      }

      a:hover {
        text-decoration: underline;
      }

      ia-button.submit-btn {
        margin-top: 10px;
      }

      @media only screen and (max-width: 992px) {
        .form-wrapper {
          max-width: 45rem;
          width: 100%;
        }
      }

      @media only screen and (max-width: 768px) {
        .form-wrapper {
          max-width: 43rem;
          width: 100%;
        }
      }

      @media only screen and (max-width: 480px) {
        .form-wrapper {
          width: 30rem;
          padding: 0 20px;
        }
      }
    `}};_([n({type:String})],C.prototype,`endpointPath`,void 0),_([n({type:Object})],C.prototype,`appServices`,void 0),_([n({type:String})],C.prototype,`referer`,void 0),_([n({type:String})],C.prototype,`prefilledUsername`,void 0),_([n({type:Object})],C.prototype,`pkceParams`,void 0),_([n({type:Object})],C.prototype,`successCallback`,void 0),_([s()],C.prototype,`pageMetadataService`,void 0),_([s()],C.prototype,`fetchHandler`,void 0),_([s()],C.prototype,`csrfToken`,void 0),_([s()],C.prototype,`googleClientId`,void 0),_([s()],C.prototype,`archiveApiUrl`,void 0),_([s()],C.prototype,`loginStatus`,void 0),_([s()],C.prototype,`loginError`,void 0),_([s()],C.prototype,`submissionInProgress`,void 0),_([s()],C.prototype,`hideIAForm`,void 0),_([m(`form`)],C.prototype,`form`,void 0),_([m(`#password-input`)],C.prototype,`passwordInput`,void 0),C=_([p(`login-page`),i()],C);var w=C;export{S as t};