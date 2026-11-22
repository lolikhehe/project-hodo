import{A as e,C as t,N as n,O as r,P as i,S as a,T as o,_ as s,w as c}from"./vendor-lit-MQ4DzmMc.js";import{a as l,o as u,t as d}from"./index-NFRVgqV2.js";import"./ia-auth-options-DIzxHnZF.js";import{t as f}from"./ia-logo-gray-circle-BADeBCkf.js";var p=class extends o{constructor(...e){super(...e),this.endpointPath=`/services/account/signup/`,this.signupStatus=`ready`,this.csrfToken=``,this.hideIAForm=!1}render(){return r`
      <div id="page-container">
        ${this.signupStatus===`success`?this.signupSuccessTemplate:this.signupFormTemplate}
      </div>
    `}willUpdate(e){e.has(`appServices`)&&this.appServices&&this.setupAppServices(),e.has(`referer`)&&this.referer&&this.validateReferer()}get signupSuccessTemplate(){return r`
      <section class="success-wrapper">
        <h1 class="success-header">${i(`Verification Email Sent`)}</h1>
        <div class="success-message">
          <p class="email-info">
            ${i(r`We've sent an email to
                <b>${this.submittedUsername??`the address you provided`}</b>.
                Please check your Spam folder if you do not see the verification
                email in your Inbox.`)}
          </p>
          <p>
            ${i(`Click on the verification link within it to finish creating your account.`)}
          </p>
          <a href="/">${i(`Continue on to the home page`)}</a>
        </div>
      </section>
    `}get signupFormTemplate(){return r`
      <section class="form-wrapper">
        ${s(!this.hideIAForm,()=>r`
            <img src=${f} alt=${i(`Internet Archive Logo`)} />
            <h1>${i(`Sign up for an account`)}</h1>
            <p class="sub-heading">
              ${i(`Get your virtual library card and access 65+ million free and borrowable books, movies, audio, images, software, and more! Already have one? `)}
              <a class="log-in-link" href="/login${this.refererParams}">
                ${i(`Log in!`)}
              </a>
            </p>
          `)}
        <ia-auth-options
          .appServices=${this.appServices}
          .csrfToken=${this.csrfToken}
          .googleClientId=${this.googleClientId??``}
          .referer=${this.referer}
          .archiveApiUrl=${this.archiveApiUrl??``}
          .hostPageName=${`sign up`}
          ?useThirdPartyTakeover=${this.hideIAForm}
          @formTakeoverRequested=${()=>this.hideIAForm=!0}
          @formTakeoverEnded=${()=>this.hideIAForm=!1}
          @IAThirdPartyAuthVerified=${()=>this.handleSuccess()}
          @otpValidated=${this.handleSuccess}
        >
          <span slot="otp-message">${i(`Email me a code to sign up`)}</span>
        </ia-auth-options>
        ${s(!this.hideIAForm,()=>r`
            <ia-signup-form
              .fetchHandler=${this.fetchHandler}
              .csrfToken=${this.csrfToken}
              .endpointPath=${this.endpointPath}
              .prefilledUsername=${this.prefilledUsername}
              ?submissionInProgress=${this.signupStatus===`submitting`}
              @formSubmitted=${this.handleFormSubmitted}
            >
              ${s(this.signupStatus===`signup_failed`,()=>r`<span slot="signup-error"
                    >${i(`Sorry, we were unable to sign you up at this time.`)}</span
                  >`)}
            </ia-signup-form>
          `)}
      </section>
    `}get refererParams(){return this.referer?`?referer=`+encodeURIComponent(this.referer):``}async setupAppServices(){let e=await this.appServices?.get();await Promise.all([e?.pageMetadataService.get().then(e=>{e.setTitle(i(`Sign Up at the Internet Archive`)),e.addLink({rel:`canonical`,path:`/signup`})}),e?.recaptchaManager.get().then(e=>{this.recaptchaManager=e,this.setupRecaptcha()}),e?.fetchHandler.get().then(e=>{this.fetchHandler=e}),e?.csrfService.get().then(async e=>this.csrfToken=await e.getToken()),e?.appConfig.get().then(e=>{this.googleClientId=e?.googleClientId,this.archiveApiUrl=e?.archiveApiUrl})])}validateReferer(){this.referer&&(u(this.referer)||(this.referer=void 0))}async handleFormSubmitted(e){this.signupStatus=`submitting`;try{let t=await this.recaptchaWidget?.execute();if((await this.fetchHandler?.fetchApiPathResponse(this.endpointPath,{includeCredentials:!0,includeCsrfToken:!0,method:`POST`,body:JSON.stringify({username:e.detail.data.username,screenname:e.detail.data.screenname,password:e.detail.data.password,notifications:e.detail.data.notifications,t:this.csrfToken,"g-recaptcha-response":t}),headers:{"Content-Type":`application/json`},retryConfig:l.noRetry}))?.success){this.submittedUsername=e.detail.data.username,this.signupStatus=`success`;return}this.signupStatus=`signup_failed`}catch{this.signupStatus=`signup_failed`}}handleSuccess(){this.redirectTo(this.referer??`/`)}redirectTo(e){window.location.href=e}async setupRecaptcha(){try{this.recaptchaWidget=await this.recaptchaManager?.getRecaptchaWidget()}catch(e){console.error(`[signup-page] reCAPTCHA widget failed to load:`,e)}}static get styles(){return e`
      :host {
        --ia-theme-secondary-background-color--: var(
          --ia-theme-secondary-background-color,
          white
        );
        --ia-theme-link-color--: var(--ia-theme-link-color, #4b64ff);
      }

      #page-container {
        background-color: var(--ia-theme-secondary-background-color--);
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

      .success-wrapper {
        margin: 0 30px;
      }

      .success-header {
        padding-top: 30px;
        text-align: left;
      }

      img {
        height: 8rem;
        margin: 3rem 0;
      }

      .sub-heading {
        margin: 1rem 0 2rem 0;
        text-align: center;
      }

      a:link,
      a:visited {
        color: var(--ia-theme-link-color--);
        text-decoration: none;
      }

      a:hover {
        text-decoration: underline;
      }

      ia-signup-form {
        padding: 0 20px;
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
    `}};d([t({type:Object})],p.prototype,`appServices`,void 0),d([t({type:String})],p.prototype,`endpointPath`,void 0),d([t({type:String})],p.prototype,`prefilledUsername`,void 0),d([t({type:String})],p.prototype,`referer`,void 0),d([a()],p.prototype,`signupStatus`,void 0),d([a()],p.prototype,`submittedUsername`,void 0),d([a()],p.prototype,`csrfToken`,void 0),d([a()],p.prototype,`googleClientId`,void 0),d([a()],p.prototype,`archiveApiUrl`,void 0),d([a()],p.prototype,`fetchHandler`,void 0),d([a()],p.prototype,`recaptchaManager`,void 0),d([a()],p.prototype,`hideIAForm`,void 0),p=d([c(`signup-page`),n()],p);var m=p;export{m as default};