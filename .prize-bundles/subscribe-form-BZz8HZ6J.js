import{A as e,C as t,E as n,N as r,O as i,P as a,S as o,T as s,l as c,w as l,x as u}from"./vendor-lit-MQ4DzmMc.js";import{t as d}from"./index-NFRVgqV2.js";import{t as f}from"./purify.es-RmFZhtLI.js";import"./ia-button-DYuVUpnO.js";import{t as p}from"./ia-logo-gray-circle-BADeBCkf.js";var m=class extends s{constructor(...e){super(...e),this.baseHost=`https://archive.org`,this.endpointPath=`/services/account/subscribe/`,this.submissionInProgress=!1,this.bypassRecaptcha=!1}get errorMessages(){return{exists:a(`Looks like you have an Archive.org account. Please Log In and go to your Account Settings to set your subscription preferences.`),invalid:a(`Please enter a valid email address.`),select:a(`Please select at least one newsletter choice.`),error:a(`Sorry, unable to subscribe you at this time. Please try again later or with a different email.`)}}render(){let e=this.errorKey?this.errorMessages[this.errorKey]:void 0;return i`<div id="page-container">
      <section class="subscribe-form">
        <img src=${p} alt=${a(`Internet Archive Logo`)} />
        <h1>${a(`Subscribe to Internet Archive Newsletters`)}</h1>
        <form
          id="subscribe-form"
          method="post"
          action="${this.baseHost}${this.endpointPath}"
        >
          <label for="email-input" class="section-heading"
            >${a(`Enter email address`)}</label
          >
          <input
            type="email"
            name="username"
            id="email-input"
            .value=${this.email??``}
            required
          />
          <legend class="section-heading">${a(`Newsletters`)}</legend>
          <div class="checkbox">
            <input
              type="checkbox"
              name="mailing_lists_checkboxes[ml_best_of]"
              id="ml_best_of"
              value="1"
            />
            <label for="ml_best_of"
              >${a(`Best of the Archive: Useful resources, unique stories, and fun finds from our collections`)}
            </label>
          </div>
          <div class="checkbox">
            <input
              type="checkbox"
              name="mailing_lists_checkboxes[ml_events]"
              id="ml_events"
              value="1"
            />
            <label for="ml_events"
              >${a(`Event Notices: Invitations to and news about our events`)}
            </label>
          </div>

          ${e?i`<div class="error">${this.renderSafeHTML(e)}</div>`:n}
          <input
            type="hidden"
            id="recaptcha-input"
            name="g-recaptcha-response"
            value=""
          />
          <ia-button
            mode="primary"
            @click=${this.handleSubmit}
            ?loading=${this.submissionInProgress}
          >
            ${a(`Subscribe`)}
          </ia-button>
        </form>
      </section>
    </div>`}updated(e){if(e.has(`appServices`)&&this.appServices&&this.setupAppServices(),e.has(`prefilledError`)&&this.prefilledError&&(this.errorKey=this.prefilledError),e.has(`prefilledValues`)&&this.prefilledValues){let e=JSON.parse(this.prefilledValues);this.subscribeForm.ml_best_of.checked=e.ml_best_of===`1`,this.subscribeForm.ml_events.checked=e.ml_events===`1`}}async setupAppServices(){await Promise.all([this.setupPageMetadata(),this.setupRecaptcha()])}async setupPageMetadata(){let e=await(await this.appServices?.get())?.pageMetadataService.get(),t=a(`Subscribe to Internet Archive Newsletters`);e?.setTitle(t),e?.addLink({rel:`canonical`,path:`/subscribe`}),e?.setMetaTag({property:`og:type`,content:`website`})}async setupRecaptcha(){let e=await this.appServices?.get(),t=(await(await e?.userService.get())?.getLoggedInUser())?.success;if(this.bypassRecaptcha=!!t&&(t?.privs.length>1||t?.privs[0]!==`0`),this.bypassRecaptcha)return;let n=await e?.recaptchaManager.get();try{this.recaptchaWidget=await n?.getRecaptchaWidget()}catch(e){console.error(`[subscribe-form] reCAPTCHA widget failed to load:`,e)}}async handleSubmit(e){if(e.preventDefault(),this.errorKey=void 0,this.submissionInProgress=!0,!this.validateFormInputs()){this.submissionInProgress=!1;return}this.bypassRecaptcha&&this.subscribeForm.requestSubmit();try{let e=await this.recaptchaWidget?.execute();e&&(this.recaptchaInput.value=e),this.subscribeForm.requestSubmit()}catch{this.errorKey=`error`,this.submissionInProgress=!1}}validateFormInputs(){return this.subscribeForm.username.value?!this.subscribeForm[`mailing_lists_checkboxes[ml_events]`].checked&&!this.subscribeForm[`mailing_lists_checkboxes[ml_best_of]`].checked?(this.errorKey=`select`,!1):!!this.subscribeForm.reportValidity():(this.errorKey=`invalid`,!1)}renderSafeHTML(e){return c(f.sanitize(e,{ALLOWED_TAGS:[`a`]}))}static get styles(){return e`
      #page-container {
        background-color: var(--ia-theme-secondary-background-color, white);
        min-height: 100vh;
        font-size: 1.4rem;
      }

      .subscribe-form {
        margin: 0 auto;
        max-width: 60rem;
        display: flex;
        flex-direction: column;
        align-items: center;
      }

      h1 {
        font-size: 2.4rem;
        font-weight: 500;
        margin-bottom: 20px;
        margin-top: 0;
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
      }

      .section-heading {
        font-size: 1.6rem;
        font-weight: bold;
      }

      input[type='email'] {
        width: 90%;
        min-width: 300px;
        padding: 3px;
        font-size: 1.6rem;
        font-family: inherit;
        margin-bottom: 10px;
      }

      .checkbox {
        display: flex;
        flex-direction: row;
        align-items: flex-start;
        gap: 0.5rem;
      }

      .error {
        color: #cc0000;
        margin-top: 1rem;
      }

      ia-button {
        margin-top: 2rem;
        align-self: center;
      }

      @media only screen and (max-width: 992px) {
        .subscribe-form {
          max-width: 45rem;
          width: 100%;
        }
      }

      @media only screen and (max-width: 768px) {
        .subscribe-form {
          max-width: 43rem;
          width: 100%;
        }
      }

      @media only screen and (max-width: 480px) {
        .subscribe-form {
          width: 30rem;
          padding: 0 20px;
        }
      }
    `}};d([t({type:String})],m.prototype,`baseHost`,void 0),d([t({type:String})],m.prototype,`endpointPath`,void 0),d([t({type:Object})],m.prototype,`appServices`,void 0),d([t({type:String})],m.prototype,`email`,void 0),d([t({type:String})],m.prototype,`prefilledError`,void 0),d([t({type:String})],m.prototype,`prefilledValues`,void 0),d([o()],m.prototype,`recaptchaWidget`,void 0),d([o()],m.prototype,`errorKey`,void 0),d([o()],m.prototype,`submissionInProgress`,void 0),d([o()],m.prototype,`bypassRecaptcha`,void 0),d([u(`#recaptcha-input`)],m.prototype,`recaptchaInput`,void 0),d([u(`#subscribe-form`)],m.prototype,`subscribeForm`,void 0),m=d([l(`subscribe-form`),r()],m);var h=m;export{h as default};