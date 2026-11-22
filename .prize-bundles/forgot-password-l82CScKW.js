import{A as e,C as t,E as n,N as r,O as i,P as a,S as o,T as s,u as c,w as l,x as u}from"./vendor-lit-MQ4DzmMc.js";import{t as d}from"./index-NFRVgqV2.js";import"./ia-button-DYuVUpnO.js";var f=class extends s{constructor(...e){super(...e),this.endpointPath=`/services/account/forgot-password/`,this.recaptchaError=!1,this.state=`form`,this.submissionInProgress=!1}get defaultErrorMsg(){return a(`Sorry, we are unable to reset your password at this time.`)}render(){return i`<div id="page-container">
      <section class="forgot-password-form">
        ${c(this.state,[[`error`,()=>this.errorStateTemplate],[`success`,()=>this.successStateTemplate]],()=>this.formTemplate)}
      </section>
    </div>`}willUpdate(e){e.has(`appServices`)&&this.appServices&&this.setupAppServices()}get errorStateTemplate(){return i`
      <h1>${a(`Error`)}</h1>
      <p class="message">${this.errorMsg??this.defaultErrorMsg}</p>
    `}get successStateTemplate(){return i`
      <h1>${a(`Done!`)}</h1>
      <p class="message">
        ${a(i`If <b>${this.email}</b> is a valid account, an email will be sent
            with instructions on resetting your password.`)}
      </p>
    `}get formTemplate(){return i`
      <h1>${a(`Reset Your Password`)}</h1>
      <p>
        ${a(`Please enter the email address associated with your Internet Archive account. We'll send you an email to help you reset your password.`)}
      </p>
      <form id="reset-password-form" @submit=${this.handleSubmit}>
        <label for="email-input">${a(`Your email address`)}</label>
        <input
          type="email"
          name="email"
          id="email-input"
          .value=${this.email??``}
          required
        />
        ${this.recaptchaError?i`<div class="error">${this.defaultErrorMsg}</div>`:n}
        <ia-button
          type="submit"
          mode="primary"
          ?loading=${this.submissionInProgress}
        >
          ${a(`Send Reset Password Email`)}
        </ia-button>
      </form>
    `}async setupAppServices(){let e=await this.appServices?.get();await Promise.all([e?.recaptchaManager.get().then(e=>{this.recaptchaManager=e,this.setupRecaptcha()}),e?.pageMetadataService.get().then(e=>{this.pageMetadataService=e,this.setupPageMetadata()}),e?.userService.get().then(e=>{this.userService=e,this.prefillEmailAddress()}),e?.fetchHandler.get().then(e=>{this.fetchHandler=e})])}async prefillEmailAddress(){let e=new URL(window.location.href).searchParams.get(`email`);if(e)this.email=e;else{let e=(await this.userService?.getLoggedInUser())?.success;this.email=e?.username??``}}async setupRecaptcha(){try{this.recaptchaWidget=await this.recaptchaManager?.getRecaptchaWidget()}catch(e){console.error(`[forgot-password] reCAPTCHA widget failed to load:`,e)}}async setupPageMetadata(){let e=a(`Reset your password`);this.pageMetadataService?.setTitle(e),this.pageMetadataService?.addLink({rel:`canonical`,path:`/forgot-password`})}async handleSubmit(e){if(e.preventDefault(),this.submissionInProgress||!this.resetPasswordForm.reportValidity())return;this.submissionInProgress=!0,this.recaptchaError=!1;let t=await this.recaptchaWidget?.execute();if(!t){this.recaptchaError=!0,this.submissionInProgress=!1;return}if(!this.fetchHandler){this.renderError();return}let n=this.resetPasswordForm.email.value;if(!n){this.renderError(a(`You must provide an email`));return}this.email=n;try{let e=await this.fetchHandler?.fetchApiPathResponse(this.endpointPath,{includeCredentials:!0,includeCsrfToken:!0,method:`PUT`,body:JSON.stringify({email:n,"g-recaptcha-response":t}),headers:{"Content-Type":`application/json`}});if(e.success){this.state=`success`,this.submissionInProgress=!1;return}else this.renderError(e.error)}catch{this.renderError()}}renderError(e){e&&(this.errorMsg=e),this.state=`error`,this.submissionInProgress=!1}static get styles(){return e`
      #page-container {
        background-color: var(--ia-theme-secondary-background-color, white);
        min-height: 100vh;
        font-size: 1.4rem;
      }

      .forgot-password-form {
        margin: 0 auto;
        padding: 30px 15px;
        max-width: 1170px;
      }

      h1 {
        font-size: 2.4rem;
        font-weight: 500;
        margin-bottom: 10px;
        margin-top: 0;
      }

      p {
        margin: 10px 0;
      }

      form {
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        gap: 5px;
      }

      label {
        font-size: 1.6rem;
        font-weight: bold;
      }

      input {
        min-width: 300px;
        padding: 3px;
        font-size: 1.6rem;
        font-family: inherit;
        margin-bottom: 10px;
      }

      .error {
        color: #cc0000;
      }

      @media only screen and (max-width: 1200px) {
        .forgot-password-form {
          max-width: 970px;
        }
      }

      @media only screen and (max-width: 992px) {
        .forgot-password-form {
          max-width: 750px;
        }
      }
    `}};d([t({type:String})],f.prototype,`endpointPath`,void 0),d([t({type:String})],f.prototype,`email`,void 0),d([t({type:Object})],f.prototype,`appServices`,void 0),d([o()],f.prototype,`pageMetadataService`,void 0),d([o()],f.prototype,`userService`,void 0),d([o()],f.prototype,`fetchHandler`,void 0),d([o()],f.prototype,`recaptchaManager`,void 0),d([o()],f.prototype,`recaptchaError`,void 0),d([o()],f.prototype,`state`,void 0),d([o()],f.prototype,`errorMsg`,void 0),d([o()],f.prototype,`submissionInProgress`,void 0),d([u(`#reset-password-form`)],f.prototype,`resetPasswordForm`,void 0),f=d([l(`forgot-password`),r()],f);var p=f;export{p as default};