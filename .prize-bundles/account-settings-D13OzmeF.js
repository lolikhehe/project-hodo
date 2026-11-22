import{A as e,C as t,E as n,F as r,N as i,O as a,P as o,S as s,T as c,l as ee,v as te,w as l,x as u}from"./vendor-lit-MQ4DzmMc.js";import{s as d}from"./vendor-shared-62wh2Xkx.js";import{t as f}from"./index-NFRVgqV2.js";import{i as p}from"./dist-BqHmxVdQ.js";import"./ia-status-indicator-Bd61y7bd.js";import{a as ne}from"./esm-CWeBglMZ.js";import{t as m}from"./log-Bvw5tZC3.js";import{t as re}from"./third-party-auth-service-C371OWBh.js";import{t as ie}from"./purify.es-RmFZhtLI.js";import"./ia-button-DYuVUpnO.js";import{a as ae,c as h,l as g,n as _,o as v,r as oe,s as y,t as se}from"./dist-DuoWfHPQ.js";import"./ia-auth-options-DIzxHnZF.js";import"./ia-pic-uploader-cgRUdxCV.js";import{t as b}from"./capitalize-BPJI7v6S.js";import{t as ce}from"./close-DnNtqaAm.js";function x(e){e?.preventDefault(),e?.stopPropagation()}function le(e,t){let n=!1;return()=>{n||(e(),n=!0,setTimeout(()=>{n=!1},t))}}var ue=e`
  :host {
    --error-color--: var(--error-color, #bb0505);
    --white--: var(--white, #fff);
    --ia-theme-link-color--: var(--ia-theme-link-color, #4b64ff);

    display: block;
    padding: 25px;
    font-size: 1.4rem;
    background-color: var(--white--);
    min-height: calc(100vh - 50px);
  }

  ia-verification {
    width: 90vw;
    max-width: 525px;
  }

  /* Buttons */
  ia-button {
    display: inline;
    margin-right: 0.5rem;
    --ia-theme-button-padding: 0 2rem;
  }

  /* General Layout */
  .settings-template {
    display: flex;
    flex-wrap: wrap;
    max-width: 700px;
  }

  .container {
    display: grid;
    justify-content: center;
    flex-direction: column;
  }

  /* Header Styles */
  .header {
    display: flex;
    flex-direction: column;
    margin-bottom: 1rem;
  }
  .header .actions {
    display: flex;
    margin-bottom: 1rem;
  }

  .header h2 {
    margin: 0 10px 0 0;
    font-size: 2.1rem;
  }

  /* Sticky Header */
  .sticky-header .header {
    position: fixed;
    top: 0;
    width: 88%;
    padding: 2rem 2rem 2rem 0;
    z-index: 2;
    background: var(--ia-theme-primary-background-color);
  }
  .sticky-header .body-content {
    margin-top: 50px;
  }

  .form-element h3 {
    margin: 0 0 1rem 0;
  }

  a,
  a:hover,
  a:focus {
    color: var(--ia-theme-link-color--);
  }

  /* Form Elements */
  .form-element {
    margin-bottom: 15px;
  }

  .pic-uploader-section label {
    display: block;
    font-weight: bold;
    margin-bottom: 4px;
  }

  /* Error and Success Messages */
  .error-field {
    color: var(--error-color--);
    font-size: 1.4rem;
    padding: 0 0.5rem;
    cursor: pointer;
    display: block;
  }

  /* Miscellaneous */
  .pointer-none {
    pointer-events: none;
    z-index: 1;
  }

  .initial-load-template {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 200px;
    padding: 2rem;
  }

  .initial-load-template p {
    margin-top: 1rem;
    font-size: 1.4rem;
    color: #333;
  }
  .initial-load-template .loader-icon {
    --icon-width: 6rem;
  }
`;function de(e){return e?.replace(/\s+/g,` `)?.trim()}function S(){return{PROFILE_PICTURE:o(`Your profile picture has been updated. Please allow 5 minutes for the change to take effect.`),GOOGLE_PROVIDER_UNLINKED:o(`Your Google account has been unlinked.`),SCREENNAME_UPDATED:o(`Your screen name has been updated successfully.`),MAILING_LISTS_UPDATED:o(`Mailing lists has been updated!`),EMAIL_UPDATED:o(`Your email has been updated successfully.`),PASSWORD_UPDATED:o(`Your password has been updated successfully.`),LOAN_HISTORY_UPDATED:o(`Your loan history visibility has been updated successfully.`),SETTINGS_SAVED:o(`Your account settings have been saved successfully.`)}}function C(){return{INVALID_EMAIL:o(`This does not appear to be a valid email address.`),EMAIL_TAKEN:e=>o(r`${e} is already taken.`),SCREENNAME_LENGTH:o(`The screen name needs to be between 3 and 127 characters long.`),INVALID_SCREENNAME:o(`This does not appear to be a valid screen name.`),SCREENNAME_TAKEN:e=>o(r`The screen name ${e} is already taken.`),PASSWORD_EMPTY:o(`* password can't be empty.`),PASSWORD_LENGTH:(e,t)=>o(r`The password needs to be between ${e} and ${t} characters long.`),API_ERROR:o(`An error occurred while processing your request.`),CSRF_TOKEN_INVALID:o(`Invalid CSRF token, please refresh the page and try again later.`),SAVE_FAILED:o(`Failed to save account settings.`),DELETE_FAILED:o(`Failed to delete account.`),FETCH_FAILED:o(`Failed to fetch account settings.`),EMAIL_CHECK_FAILED:o(`Failed to check email availability.`),SCREENNAME_CHECK_FAILED:o(`Failed to check screenname availability.`),UPLOAD_FAILED_GENERIC:o(`Failed to change your profile picture.`),GOOGLE_PROVIDER_UNLINK_FAILED:o(`Failed to unlink your google account.`)}}var fe=class{constructor(e,t){this.userData={email:``,screenname:``,password:``,loanHistoryFlag:``},this.existingUserData={email:``,screenname:``,password:``},this.selectedMailingLists=[],this.csrfToken=``,this.fieldsError={},this.passwordMinLength=3,this.passwordMaxLength=100,this.settingsServicePath=``,this.preferenceMap={},this.initialPreferenceMap={},this.selectedPreferenceSettings={},this.settingsServicePath=e,this.fetchHandler=t}hasFieldError(e=this.fieldsError){return Object.values(e).some(e=>e!==``)}handleSetEmail(e){this.userData={...this.userData,email:e},this.fieldsError={...this.fieldsError,email:``}}handleSetScreenname(e){this.userData={...this.userData,screenname:e},this.fieldsError={...this.fieldsError,screenname:``}}handleSetPassword(e){this.userData={...this.userData,password:e},this.fieldsError={...this.fieldsError,password:``}}handleSettingsChanges(e){let{category:t,key:n,value:r}=e.detail;t===`account`?(n===`screenname`||n===`email`||n===`password`)&&this.updateInputField(n,r):[`search`,`display`].includes(t)?this.updatePreferences(t,n,r):t===`mailing`&&this.updateMailingLists(n,r)}updateInputField(e,t){e===`screenname`?(this.handleSetScreenname(t),this.fieldsError={...this.fieldsError,screenname:``}):e===`email`?(this.handleSetEmail(t),this.fieldsError={...this.fieldsError,email:``}):e===`password`&&(this.handleSetPassword(t),this.fieldsError={...this.fieldsError,password:``})}updatePreferences(e,t,n){let r=`${e}_${t}`,i=n===!0?`on`:n===!1?`off`:n;this.initialPreferenceMap[r]===i?delete this.selectedPreferenceSettings[r]:this.selectedPreferenceSettings[r]=i,m(`Current preferences changes:`,this.selectedPreferenceSettings)}updateMailingLists(e,t){t===`on`&&!this.selectedMailingLists.includes(e)?this.selectedMailingLists.push(e):t===`off`&&this.selectedMailingLists.includes(e)&&(this.selectedMailingLists=this.selectedMailingLists.filter(t=>t!==e)),this.selectedMailingLists?.sort(),m(`Current mailing lists:`,this.selectedMailingLists)}async validateEmail(){let e=de(this.userData.email),t=/^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*$/,n=``;return e?.match(t)?await this.isEmailAvailable(e)||(n=C().EMAIL_TAKEN(e)):n=C().INVALID_EMAIL,this.fieldsError.email=n,n}async validateScreenname(){let e=de(this.userData.screenname),t=e.length<3||e.length>127,n=``;return e===``||t?n=C().SCREENNAME_LENGTH:e?.includes(`\\`)?n=C().INVALID_SCREENNAME:await this.isScreennameAvailable(e)||(n=C().SCREENNAME_TAKEN(e)),this.fieldsError.screenname=n,n}validatePassword(){let e=this.userData?.password?.length??0,t=``;return e>0&&(e<this.passwordMinLength||e>this.passwordMaxLength)&&(t=C().PASSWORD_LENGTH(this.passwordMinLength,this.passwordMaxLength)),this.fieldsError.password=t,t}async isEmailAvailable(e){try{if(e!==this.existingUserData?.email)return!!(await this.fetchHandlerHelper({action:`email-available`,email:e},`GET`)).success}catch(e){return m(`Error checking email availability:`,e),!1}return!0}async isScreennameAvailable(e){try{if(e!==this.existingUserData.screenname)return!!(await this.fetchHandlerHelper({action:`screenname-available`,screenname:e},`GET`)).success}catch(e){return m(`Error checking screenname availability:`,e),!1}return!0}async handleSaveAccountSettings(){try{let e=Object.keys(this.selectedPreferenceSettings).length>0,t={action:`save-account`,identifier:this.userData.identifier,userdata:JSON.stringify(this.userData),"selected-mailing-lists":this.selectedMailingLists?.join(`,`),...e&&{"selected-preference-settings":JSON.stringify(this.selectedPreferenceSettings)}},n=await this.fetchHandlerHelper(t);return this.initialPreferenceMap=this.selectedPreferenceSettings,this.selectedPreferenceSettings={},this.userData.password=``,n}catch(e){throw m(`Error saving account settings:`,e),{success:!1,error:e}}}async unlinkProvider(e){try{return await new re(this.csrfToken,this.fetchHandler).unlinkUserFromAccount({provider:e,providerToken:``})}catch(e){return m(`Error unlinking provider:`,e),{success:!1}}}async deleteAccount(e){try{return await this.fetchHandlerHelper({action:`delete-account`,"delete-confirm":e},`DELETE`)}catch(e){throw m(`Error deleting account:`,e),{success:!1,error:C().DELETE_FAILED}}}async fetchAccountSettings(){try{let e=await this.fetchHandler?.fetchApiPathResponse(this.settingsServicePath,{includeCredentials:!0});if(!e.success)throw Error(`Failed to fetch account settings`);return this.userData=e?.value?.userData,this.csrfToken=e?.value?.csrfToken??``,this.existingUserData=this.userData,this.mapPreferences(e?.value?.preferenceSettings||[]),this.mapMailingLists(e.value?.selectedMailingLists||[]),e}catch{throw Error(`Failed to fetch account settings`)}}mapPreferences(e){this.preferenceMap={},e.forEach(e=>{let t=`${e.category}_${e.preference}`;this.preferenceMap[t]=e.value}),this.initialPreferenceMap={...this.preferenceMap}}mapMailingLists(e){this.selectedMailingLists=[...e]}async fetchHandlerHelper(e,t=`POST`){let n={};Object.entries(e).forEach(([e,t])=>{t!=null&&(n[e]=t.toString())});let r={method:t,includeCredentials:!0,includeCsrfToken:!0};try{t===`GET`||t===`DELETE`?r.queryParams=n:t===`POST`&&(r.body=JSON.stringify(n),r.headers={"Content-Type":`application/json`});let e=await this.fetchHandler?.fetchApiPathResponse(this.settingsServicePath,r);return m(`Fetch response:`,e),e}catch(e){throw Error(`Failed to process the request ${e}`)}}},w=`AccountSettings`,T=function(e){return e.authenticated=`AuthOK`,e.deleteAccount=`DeleteAccount`,e.pageView=`PageView`,e.saved=`Saved`,e.saveFailed=`SaveFailed`,e}({}),E=class{constructor(e){this.paymentMethodInfo=e.paymentMethodInfo,this.donorContactInfo=e.donorContactInfo,this.paymentProvider=e.paymentProvider??`unknown`}},pe=`mailto:donations@archive.org?subject=Please%20Confirm%20my%20Monthly%20Giving%20Circle%20Status&body=Dear%20Philanthropy%20Team%2C%0A%0AI%20am%20already%20a%20member%20of%20the%20Monthly%20Giving%20Circle%2C%20but%20I%20do%20not%20see%20my%20recurring%20donation%20under%20Account%20Settings.%20Please%20use%20the%20following%20information%20to%20locate%20my%20donation%3A%0A%0AFull%20name%3A%20____________%0A%0AEmail%20address(es)%3A%20____________%0A%0A*Please%20provide%20any%20alternative%20email%20addresses%20you%20may%20have%20used%20to%20create%20your%20recurring%20donation.%0A%0AIf%20you%20have%20access%20to%20a%20monthly%20donation%20receipt%2C%20please%20include%20a%20screenshot%20with%20your%20email.%0A%0AThanks!`,D=class extends c{constructor(){super(...arguments),this.patronName=``,this.baseHost=`archive.org`,this.defaultMonthlyDonationAmount=5}render(){return a`
      <section>
        <p>
          Join the Monthly Giving Circle by starting a monthly recurring
          donation at any level to access benefits and perks:
        </p>
        <ul>
          <li>Exclusive webinars and virtual learning opportunities</li>
          <li>Discounts at Better World Books</li>
          <li>
            Access to the curated Monthly Giving Circle newsletter and more!
          </li>
        </ul>

        <a href=${`https://${this.baseHost}/donate/?amt=${this.defaultMonthlyDonationAmount}&contrib_type=monthly&origin=iawww-usrsttng`} target="_blank" class="link join-mgc"
          >Click here to join the Monthly Giving Circle</a
        >
        <p>
          Already a monthly donor but don't see your donation details here?
          <br />
          <a class="link" href=${pe}
            >Contact us at donations@archive.org</a
          >
        </p>
      </section>
    `}get mailToInquiryLink(){return`mailto:donations@archive.org?subject=Tell%20me%20more%20about%20the%20Monthly%20Giving%20Circle&body=Hello%2C%20I%20am%20an%20archive%20member%20and%20want%20to%20know%20about%20the%20Monthly%20Giving%20Circle.%20%20Best%2C%20${encodeURIComponent(this.patronName)}`}};D.styles=e`
    :host {
      border: 1px solid #23765d;
      background-color: #eeffee;
      display: block;
      width: inherit;
    }

    section {
      margin: 0.5rem;
    }

    section > p {
      margin: 0;
    }

    section a.join-mgc {
      padding-bottom: 1rem;
      display: block;
    }

    ul {
      list-style-type: disc;
      padding-left: 1rem;
      margin-left: 1rem;
    }
  `,d([t({type:String})],D.prototype,`patronName`,void 0),d([t({type:String})],D.prototype,`baseHost`,void 0),d([t({type:String})],D.prototype,`defaultMonthlyDonationAmount`,void 0),D=d([l(`ia-mgc-welcome`)],D);var me=a`
<svg
  viewBox="0 0 40 40"
  xmlns="http://www.w3.org/2000/svg"
  aria-labelledby="donateTitleID donateDescID"
>
  <title id="donateTitleID">Donate icon</title>
  <desc id="donateDescID">An illustration of a heart shape</desc>
  <path class="fill-color" d="m30.0120362 11.0857287c-1.2990268-1.12627221-2.8599641-1.65258786-4.682812-1.57894699-.8253588.02475323-1.7674318.3849128-2.8262192 1.08047869-1.0587873.6955659-1.89622 1.5724492-2.512298 2.63065-.591311-1.0588196-1.4194561-1.9357029-2.4844351-2.63065-1.0649791-.69494706-2.0039563-1.05510663-2.8169316-1.08047869-1.2067699-.04950647-2.318187.17203498-3.3342513.66462439-1.0160643.4925893-1.82594378 1.2002224-2.42963831 2.1228992-.60369453.9226769-.91173353 1.9629315-.92411701 3.1207641-.03715043 1.9202322.70183359 3.7665141 2.21695202 5.5388457 1.2067699 1.4035084 2.912594 3.1606786 5.1174721 5.2715107 2.2048782 2.1108321 3.7565279 3.5356901 4.6549492 4.2745742.8253588-.6646243 2.355647-2.0647292 4.5908647-4.2003145s3.9747867-3.9171994 5.218707-5.3448422c1.502735-1.7723316 2.2355273-3.6186135 2.1983769-5.5388457-.0256957-1.7608832-.6875926-3.2039968-1.9866194-4.3302689z"/>
</svg>
`,he=class extends c{static get styles(){return e`
      :host {
        width: var(--iconWidth, 'auto');
        height: var(--iconHeight, 'auto');
      }

      .fill-color {
        fill: var(--iconFillColor);
      }

      .stroke-color {
        stroke: var(--iconStrokeColor);
      }
    `}render(){return me}};customElements.define(`ia-icon-donate`,he);var O=class extends c{constructor(){super(...arguments),this.isDisabled=!1}render(){return a`
      <button
        ?disabled=${this.isDisabled}
        @click=${e=>{this.clickHandler&&(this===null||this===void 0||this.clickHandler(e,this))}}
      >
        <slot></slot>
      </button>
    `}};O.styles=e`
    :host {
      display: inline-block;
      height: var(--button-height, 30px);
    }

    button {
      border: var(--button-border, yellow);
      background-color: var(--button-background-color, unset);
      border-radius: var(--button-border-radius, 0);

      cursor: pointer;
      line-height: normal;
      border-radius: 0.4rem;
      text-align: center;
      vertical-align: middle;
      display: inline-block;
      padding: 0.6rem 1.2rem;
      border: 1px solid transparent;

      white-space: nowrap;
      -webkit-user-select: none;
      -moz-user-select: none;
      -ms-user-select: none;
      -o-user-select: none;
      user-select: none;
    }

    :host(.transparent) button {
      background-color: transparent;
    }

    :host(.slim) button {
      padding: 0;
    }

    :host(.primary) button {
      color: #ffeeee;
      background-color: #194880;
      border-color: #c5d1df;
    }

    :host(.secondary) button {
      background: #333;
      color: #ffeeee;
    }

    :host(.cancel) button {
      border-color: #f18286;
      background-color: #d9534f;
      color: #fff;
    }

    :host(.link) button {
      color: var(--link-button-color, #4b64ff);
      border: none;
      background: transparent;
      display: flex;
      align-items: var(--link-button-flex-align-items, flex-end);
      padding: var(--link-button-padding, inherit);
      height: inherit;
    }

    :host(.link.cancel) button {
      color: var(--link-cancel-color, #bb0505);
    }

    :host(.text) button {
      color: #222;
    }

    :host(.clear-container) button {
      height: inherit !important;
      background: transparent;
    }

    :host([isdisabled]) button {
      cursor: not-allowed;
      opacity: var(--disabled-button-opacity, 0.5);
      background-color: var(--primaryDisableCTAFill, #767676);
      border: 1px solid var(--secondaryCTABorder, #999);
    }
  `,d([t({type:Boolean,reflect:!0})],O.prototype,`isDisabled`,void 0),d([t({type:Object})],O.prototype,`clickHandler`,void 0),d([u(`button`)],O.prototype,`button`,void 0),O=d([l(`ia-mgc-button`)],O);var k=class extends c{constructor(){super(...arguments),this.plans=[],this.canEdit=!0}render(){return a`
      <section class="monthly-giving-circle">
        <ul>
          ${this.plans.map(e=>{let t=e.payment?.paymentMethodType??`Method not found`;t===`creditCard`&&(t=`Credit card`);let r=e.payment?.cardType??`Card type not found`,i=e.payment?.last4?`...${e.payment?.last4}`:`CC number not found`;return a`
              <li class=${`${e.plan.isCancelled?`cancelled`:``}`}>
                <div class="info">
                  <div class="amount">
                    <h3>Amount</h3>
                    <p>${e.currency} ${e.amountFormatted}/month</p>
                    ${e.isTest?a`<p class="is-test">(Test payment)</p>`:n}
                  </div>
                  <div class="payment-details">
                    <h3>Method</h3>
                    <p>${t}</p>
                    ${e.payment?.cardType&&e.payment?.last4?a`<p>${r} ${i}</p>`:n}
                    ${e.payment?.paymentMethodType===g.PayPal?a`<p>
                          Paypal email:
                          <a href=${`mailto:${e.payment?.paypalEmail}`}
                            >${e.payment?.paypalEmail}</a
                          >
                        </p>`:n}
                    ${e.payment?.paymentMethodType===g.Venmo?a`<p>
                          Venmo username:
                          <a href=${`mailto:${e.payment?.venmoUsername}`}
                            >${e.payment?.venmoUsername}</a
                          >
                        </p>`:n}
                    ${e.payment?.expirationMonth&&e.payment?.expirationYear?a`<p>
                          Expires:
                          ${e.payment.expirationMonth}/${e.payment.expirationYear}
                        </p>`:n}
                  </div>
                  <div class="next-donation">
                    <h3>Next Donation</h3>
                    <p>${e.nextBillingDateLocale}</p>
                  </div>
                </div>
                ${this.planCTA(e)}
              </li>
            `})}
        </ul>
      </section>
    `}planCTA(e){if(!this.canEdit)return a`<p class="email-edit-plan">
        Need to update your plan further? Please email us at
        <a href=${`mailto:donations@archive.org?subject=I'd like to update my monthly donation`}>donations@archive.org</a>.
      </p>`;let t=e.plan.isCancelled?`Plan is cancelled`:`Manage this monthly donation`;return a`<ia-mgc-button
      class="ia-button link edit-donation"
      .isDisabled=${e.plan.isCancelled}
      .clickHandler=${async(t,n)=>{n.isDisabled=!0,this.dispatchEvent(new CustomEvent(`editThisPlan`,{detail:{plan:e}}))}}
      >${t}</ia-mgc-button
    >`}};k.styles=e`
    :host {
      max-height: 500px;
      overflow-y: auto;
      display: block;
    }

    .is-test {
      font-size: 0.8rem;
    }

    li {
      border: 1px solid #23765d;
      background-color: #eeffee;
      display: block;
      width: inherit;
    }
    li.cancelled {
      background-color: lightgoldenrodyellow;
      --primaryDisableCTAFill: transparent;
      --secondaryCTABorder: none;
      /* read "Plan is cancelled" as plain status text, in the card's own color */
      --link-button-color: currentColor;
      --disabled-button-opacity: 1;
    }
    table {
      width: 100%;
      text-align: left;
      max-width: 600px;
    }

    ul {
      padding: 0;
      list-style-type: none;
      margin: 0;
    }

    ul li {
      border: 1px solid #23765d;
      background-color: #eeffee;
      margin: 0.5rem 0;
      padding: 0.5rem 0.5rem 1rem 0.5rem;
      position: relative;
    }

    ul li button.edit-donation {
      height: 30px;
      display: block;
      position: absolute;
      bottom: 0;
    }

    ul li .info {
      display: grid;
      min-height: 90px;
      grid-template-columns: 0.5fr 1fr 0.5fr;
      grid-template-rows: 1fr;
      gap: 0px 5px;
      grid-auto-flow: row;
      grid-template-areas: 'amount details next-donation';
    }

    ul li .info .amount {
      grid-area: amount;
    }

    ul li .info .payment-details {
      grid-area: details;
    }

    ul li .info .next-donation {
      grid-area: next-donation;
    }

    ul li .info > * {
      margin: 0 0 0.5rem 0;
    }

    ul li .info > * > * {
      margin: 0;
    }

    .email-edit-plan {
      margin-bottom: 0;
    }

    @media screen and (max-width: 500px) {
      ul li .info {
        display: block;
      }
    }
  `,d([t({type:Array})],k.prototype,`plans`,void 0),d([t({type:Boolean,reflect:!0})],k.prototype,`canEdit`,void 0),k=d([l(`ia-mgc-plans`)],k);var A=class extends c{constructor(){super(...arguments),this.titleStyle=`heart`}get heart(){return this.titleStyle===`heart`?a` <div class="icon-donate"><ia-icon-donate></ia-icon-donate></div> `:n}render(){return a`
      <div>
        <h2>
          <div class="title-section">
            ${this.heart}
            <slot name="title"></slot>
          </div>
          <slot name="action"></slot>
        </h2>
      </div>
    `}};A.styles=e`
    :host {
      padding-bottom: 5px;
      display: block;
      --link-button-padding: 0.5rem;
    }

    :host([titlestyle='default']) h2 {
      justify-content: flex-start;
      gap: 20px;
    }

    :host([titlestyle='heart']) h2 .title-section {
      width: 100%;
      display: flex;
    }

    h2 {
      font-size: 1.5em;
      display: flex;
      flex-direction: row;
      flex-wrap: nowrap;
      align-content: center;
      margin: 0;
      justify-content: space-between;
      align-items: center;
      line-height: normal;
    }

    h2 .icon-donate {
      display: flex;
      flex-direction: row;
      flex-wrap: nowrap;
      align-content: flex-end;
      align-items: center;
      justify-content: flex-start;
    }

    h2 .icon-donate {
      --iconFillColor: red;
      --iconWidth: 1.2em;
      --iconHeight: 1.2em;
    }

    slot[name='action'] {
      padding: 0 !important;
      display: flex;
      justify-content: flex-end;
      align-items: flex-end;
    }

    ia-mgc-button {
      --link-button-padding: 0.5rem;
    }

    @media screen and (max-width: 450px) {
      h2,
      slot[name='action'] {
        display: block;
      }
      slot[name='action'] {
        margin-top: 10px;
      }
    }
  `,d([t({type:String})],A.prototype,`titleStyle`,void 0),A=d([l(`ia-mgc-title`)],A);var j=class extends c{constructor(){super(...arguments),this.receipts=[],this.receiptDispatcher=null}shouldUpdate(e){return!!e.has(`receiptDispatcher`)}updated(e){e.has(`receipts`)&&this.updateReceiptSentMap(),e.has(`receiptDispatcher`)&&console.log(`receiptDispatcher UPDATED ---- `,this.receiptDispatcher)}updateReceiptSentMap(){if(!this.receipts.length)this.receiptDispatcher=null;else{let e={};this.receipts.forEach(t=>{e[t.id]={id:t.id,emailStatus:``}}),this.receiptDispatcher=e}}emailReceipt(e){this.dispatchEvent(new CustomEvent(`EmailReceiptRequest`,{detail:{donation:e}}))}async emailSent(e){let t=this.receiptDispatcher;this.receiptDispatcher=null,await this.updateComplete;let n={...t},{id:r}=e;n[r]=e,this.receiptDispatcher={...n},console.log(`RECEIPTS -- emailSent`,this.receiptDispatcher,e);let i=CSS.escape(r),a=this.shadowRoot?.querySelector(`#donation-${i} ia-mgc-button`);a.isDisabled=!1}emailStatusMessageToDisplay(e){switch(e.emailStatus){case`success`:return`Sent`;case`fail`:return`Failed`;default:return``}}ctaButtonText(e,t){return t?.emailStatus===`pending`?`Sending...`:`Email receipt`}render(){return a`
      <section id="recent-donations-list">
        <table>
          <tr>
            <th class="date">Date</th>
            <th class="amount">Amount</th>
            <th class="action">Action</th>
          </tr>
          ${this.receipts.length?this.receipts.map(e=>{let t=this.receiptDispatcher?.[e.id],r=t?.emailStatus===`pending`,i=!t||!t.emailStatus?n:a`<ia-mgc-update-status
                        .status=${t.emailStatus}
                        >${this.emailStatusMessageToDisplay(t)}</ia-mgc-update-status
                      >`;return a`
                  <tr
                    id=${`donation-${e.id}`}
                    class=${`${e.isTest?`test`:``}`}
                  >
                    <td>
                      <div class="donation-date">${e.date}</div>
                    </td>
                    <td>
                      <div class="donation-amount">${e.amount}</div>
                    </td>
                    <td>
                      <div class="request-receipt">
                        <ia-mgc-button
                          class="link slim"
                          style="--link-button-flex-align-items: center;"
                          .clickHandler=${async(t,n)=>{if(r||(n.isDisabled=!0,await n.updateComplete),!r&&(this.emailReceipt(e),this.receiptDispatcher)){let t={...this.receiptDispatcher};t[e.id].emailStatus=`pending`,this.receiptDispatcher=t}}}
                        >
                          ${this.ctaButtonText(e,t)}
                        </ia-mgc-button>
                        ${i}
                      </div>
                    </td>
                  </tr>
                `}):a`<p>No recent donations found</p>`}
        </table>
      </section>
    `}};j.styles=e`
    table {
      text-align: left;
      table-layout: fixed;
      min-width: 600px;
    }

    button {
      padding: 1rem 0;
    }

    td {
      padding: 0;
    }

    th.date {
      width: 55px;
    }
    th.amount {
      width: 55px;
    }
    th.action {
      width: 200px;
    }
    ia-mgc-button-style {
      display: inline-block;
    }

    .request-receipt {
      display: flex;
      flex-wrap: nowrap;
      align-content: center;
      justify-content: flex-start;
      align-items: center;
      gap: 10px;
    }
  `,d([t({type:Array})],j.prototype,`receipts`,void 0),d([t({type:Object})],j.prototype,`receiptDispatcher`,void 0),j=d([l(`ia-mgc-receipts`)],j);var ge=class{constructor(e){this.storage=e??this.resolveStorage()}setPending(e){var t;let n={planId:e,timestamp:Date.now()};try{(t=this.storage)==null||t.setItem(`venmo_mgc_pending_${e}`,JSON.stringify(n))}catch{}}getPending(e){let t=null;try{t=this.storage?.getItem(`venmo_mgc_pending_${e}`)??null}catch{return null}if(!t)return null;try{let n=JSON.parse(t);return Date.now()-n.timestamp>36e5?(this.clearPending(e),null):n}catch{return this.clearPending(e),null}}clearPending(e){var t;try{(t=this.storage)==null||t.removeItem(`venmo_mgc_pending_${e}`)}catch{}}resolveStorage(){for(let e of[localStorage,sessionStorage])try{return e.setItem(`__venmo_test__`,`1`),e.removeItem(`__venmo_test__`),e}catch{}}},_e={USD:`en-US`},ve={USD:`$`};function M(e,t=`USD`){if(!Number.isFinite(e))return`Invalid amount`;let n=_e[t]??`en-US`;return new Intl.NumberFormat(n,{style:`currency`,currency:t,minimumFractionDigits:2,maximumFractionDigits:2}).format(e)}var N=class extends c{render(){return a`
      <div class="mgc-donation-section-info">
        <span><slot></slot></span>
        <ia-mgc-button
          class="ia-button link"
          .clickHandler=${(e,t)=>{t.isDisabled=!0,this.dispatchEvent(new Event(`editingRequested`))}}
        >
          Edit...
        </ia-mgc-button>
      </div>
    `}};N.styles=e`
    span {
      max-width: 300px;
      margin-right: 10px;
      display: inline-grid;
    }
  `,N=d([l(`ia-mgc-form-section-info`)],N);var P=class extends c{constructor(){super(...arguments),this.status=``}get glyph(){return this.status===`success`?`✓`:this.status===`fail`?`✖`:``}render(){return this.status?a`
      <span class="sent-status ${this.status}"
        >${this.glyph} <slot></slot
      ></span>
    `:n}};P.styles=e`
    .sent-status.success,
    .sent-status.fail {
      padding: 5px;
      background: rgb(238, 253, 238);
      width: 55px;
      min-height: 20px;
    }
    .sent-status.success {
      color: var(--mgc-success-color, rgb(33, 149, 24));
      border-left: 5px solid var(--mgc-success-color, rgb(33, 149, 24));
    }
    .sent-status.fail {
      color: var(--mgc-warning-color-dark, #bb0505);
      border-left: 5px solid var(--mgc-warning-color-dark, #bb0505);
    }
  `,d([t({type:String})],P.prototype,`status`,void 0),P=d([l(`ia-mgc-update-status`)],P);var F=class extends c{constructor(){super(...arguments),this.newAmount=0,this.currentlyEditing=!1,this.coverFees=!1,this.updateMessage=``,this.errorMessage=``,this.updateStatus=``}updated(e){e.has(`plan`)&&this.plan&&this.captureAmountChanges(),e.has(`coverFees`)&&this.captureAmountChanges(),e.has(`currentlyEditing`)&&this.currentlyEditing&&this.form.focus(),e.has(`donationPaymentInfo`)&&!this.donationPaymentInfo&&this.captureAmountChanges()}render(){return a`
      <section>
        <donation-form-section badgemode="hidebadge" headline="Amount">
          ${this.currentlyEditing?n:a`<ia-mgc-form-section-info
                @editingRequested=${()=>{this.currentlyEditing=!0,this.clearStatusMessaging()}}
                ><span
                  >${this.plan?.currency} ${this.plan?.amountFormatted}</span
                >
              </ia-mgc-form-section-info>`}
          ${this.currentlyEditing?this.editAmountForm:n}
        </donation-form-section>
      </section>
    `}async amountUpdated(e){if(this.clearInputField(),this.updateStatus=e,this.updateMessage=e===`success`?`Amount updated`:`Failed to update date, please try again`,e===`success`){this.closeForm(),await this.updateComplete,this.updateStatus=e,this.updateMessage=`Amount updated`;return}this.form.querySelector(`ia-mgc-button#update-amount`).isDisabled=!1,this.captureAmountChanges(),await this.updateComplete}requestAmountUpdate(e){e.preventDefault();let t=this.form.querySelector(`input[name="amount"]`);this.newAmount=Number(t.value),this.captureAmountChanges(this.newAmount),console.log(`<plan-amount> - update amount`,{newValue:this.newAmount,oldValue:this.plan?.plan.amount,display:this.donationPaymentInfo?.amount});let n=Number(h.calculateTotal(Number(this.donationPaymentInfo?.amount),this.coverFees));this.dispatchEvent(new CustomEvent(`updateAmount`,{detail:{plan:this.plan,amount:n,baseAmount:this.donationPaymentInfo?.amount,coverFees:this.coverFees,feeCovered:this.donationPaymentInfo?.feeAmountCovered}}))}captureAmountChanges(e){if(!this.donationPaymentInfo&&this.plan){this.donationPaymentInfo=new h({donationType:y.Monthly,amount:0,coverFees:!0});return}let t;if(e)t=new h({donationType:y.Monthly,amount:e,coverFees:!0});else{let e=this.donationPaymentInfo?this.donationPaymentInfo.amount:this.plan.amount;t=new h({donationType:y.Monthly,amount:e,coverFees:!0})}this.donationPaymentInfo=t}closeForm(){this.clearInputField(),this.clearStatusMessaging(),this.currentlyEditing=!1,this.coverFees=!1,this.errorMessage=``}clearInputField(){let e=this.form.querySelector(`input[name="amount"]`);e.value=``,this.newAmount=0,this.donationPaymentInfo=void 0}handleSubmit(e,t){e.preventDefault(),this.clearStatusMessaging();let n=t??this.form.querySelector(`#update-amount`);if(n&&this.updateButtonState(n),!this.newAmount){this.errorMessage=`Please enter a new amount`,n&&(n.isDisabled=!1);return}let r=this.form.querySelector(`input[name="amount"]`),i=Number(r.value)??0,a=i<1,o=i>=9999;if(a&&(this.errorMessage=`Please enter a valid amount`),o&&(this.errorMessage=`Amount must be less than $9,999. Would you like to donate more? Please contact us at donations@archive.org`),o||a){n&&this.updateButtonState(n);return}this.requestAmountUpdate(e)}async updateButtonState(e){e.isDisabled=!0,await e?.updateComplete}async clearStatusMessaging(){this.errorMessage=``,this.updateMessage=``,this.updateStatus=``,await this.updateComplete}totalAmountWithFees(){return this.newAmount===0?0:h.calculateTotal(this.newAmount,this.coverFees)}get coveredFeesText(){return this.newAmount?`I'll generously add ${M(h.calculateFeeAmount(this.newAmount))} to cover fees.`:`I'll generously cover the fees.`}get editAmountForm(){return a`
      <section>
        <form
          id="edit-plan-amount"
          @submit=${e=>this.handleSubmit(e)}
        >
          <p>
            Current donation amount: ${this.plan?.currency}
            ${this.plan?.amountFormatted}
          </p>
          <div>
            ${ve[this.plan?.currency??`USD`]}
            <input
              min="1"
              max="9999"
              type="number"
              id="amount"
              name="amount"
              ?required=${!0}
              @focus=${()=>this.clearStatusMessaging()}
              @input=${e=>{let t=Number(e.target.value);this.captureAmountChanges(t),this.newAmount=t}}
            />
            / month
          </div>
          <div>
            <div class="checkbox-option-container">
              <input
                type="checkbox"
                id="cover-fees"
                tabindex="0"
                @change=${e=>{let t=e.target.checked;this.coverFees=t,this.captureAmountChanges()}}
              />
              <label for="cover-fees">${this.coveredFeesText}</label>
            </div>
            <p>
              Total: ${this.plan?.currency}
              ${M(this.totalAmountWithFees())}
            </p>
            <div class="cta-container">
              <ia-mgc-button
                class="ia-button secondary"
                .clickHandler=${e=>{e.preventDefault(),this.closeForm()}}
              >
                Cancel
              </ia-mgc-button>
              <ia-mgc-button
                id="update-amount"
                class="ia-button primary"
                type="submit"
                ?isDisabled=${!this.newAmount}
                .clickHandler=${(e,t)=>this.handleSubmit(e,t)}
              >
                Update
              </ia-mgc-button>
              <ia-mgc-update-status .status=${this.updateStatus}
                >${this.updateMessage}</ia-mgc-update-status
              >
            </div>
          </div>
          <p class="error">${this.errorMessage}</p>
        </form>
      </section>
    `}};F.styles=e`
    input#amount {
      width: 50px;
    }

    .cta-container {
      display: flex;
      flex-direction: row;
      flex-wrap: wrap;
      align-content: center;
      justify-content: flex-start;
      align-items: center;
      gap: 10px;
    }

    p.error {
      color: var(--mgc-warning-color-dark, #bb0505);
    }

    ia-mgc-update-status {
      display: inline-block;
      margin-left: 1rem;
    }

    .checkbox-option-container {
      margin: 10px 0;
      display: flex;
    }
  `,d([t({type:Object})],F.prototype,`plan`,void 0),d([t({type:Object})],F.prototype,`donationPaymentInfo`,void 0),d([t({type:Number})],F.prototype,`newAmount`,void 0),d([t({type:Boolean,reflect:!0})],F.prototype,`currentlyEditing`,void 0),d([t({type:Boolean})],F.prototype,`coverFees`,void 0),d([t({type:String})],F.prototype,`updateMessage`,void 0),d([t({type:String})],F.prototype,`errorMessage`,void 0),d([t({type:String})],F.prototype,`updateStatus`,void 0),d([u(`form`)],F.prototype,`form`,void 0),F=d([l(`ia-mgc-edit-plan-amount`)],F);var I;(function(e){e.invalid_date=`Please enter a valid date format (YYYY-MM-DD)`,e.date_too_early=`Date must be at least tomorrow.`,e.second_donation_this_month=`The date you selected will result in an additional donation for this month.`,e.date_out_of_range=`New donation date must be within the next 12 months.`,e.same_next_billing_date=``})(I||(I={}));var L=class extends c{constructor(){super(...arguments),this.currentlyEditing=!1,this.allowEditing=!1,this.updateMessage=``,this.errorMessage=``,this.warningMessage=``,this.updateStatus=``}willUpdate(e){this.dateInput&&e.has(`plan`)&&this.plan&&(this.dateInput.setAttribute(`min`,this.minDate),this.dateInput.setAttribute(`max`,this.maxDate))}updated(e){e.has(`currentlyEditing`)&&this.currentlyEditing&&this.dateInput.focus()}render(){return a`
      <section>
        <donation-form-section
          badgemode="hidebadge"
          headline="Change next donation date"
        >
          ${this.currentlyEditing?n:a`<ia-mgc-form-section-info
                @editingRequested=${()=>{this.currentlyEditing=!0,this.clearStatusMessaging()}}
                ><span>${this.plan?.nextBillingDateLocale}</span>
              </ia-mgc-form-section-info>`}
          ${this.currentlyEditing?this.editDateForm:n}
        </donation-form-section>
      </section>
    `}async dateUpdated(e){if(this.clearInputField(),this.updateStatus=e,this.updateMessage=e===`success`?`Date updated`:`Failed to update date, please try again`,e===`success`){this.closeForm(),await this.updateComplete;return}this.allowEditing=!1,await this.updateComplete}requestDateUpdate(e){e.preventDefault(),this.dispatchEvent(new CustomEvent(`updateDate`,{detail:{plan:this.plan,newDate:this.newDate}}))}closeForm(){this.clearInputField(),this.clearStatusMessaging(),this.currentlyEditing=!1,this.allowEditing=!1}clearInputField(){this.dateInput.value=``,this.newDate=void 0}formatDateToYYYYMMDD(e){return`${e.getFullYear()}-${String(e.getMonth()+1).padStart(2,`0`)}-${String(e.getDate()).padStart(2,`0`)}`}async clearStatusMessaging(){this.errorMessage=``,this.warningMessage=``,this.updateMessage=``,this.updateStatus=``,await this.updateComplete}validateChosenDate(e){if(!e)return{valid:!1,errorCode:`invalid_date`};let t;try{t=new Date(`${e}T00:00:00`)}catch{return{valid:!1,errorCode:`invalid_date`}}let n=new Date;n.setHours(0,0,0,0),new Date(n).setDate(n.getDate()+1);let r=new Date(t);if(r.setHours(0,0,0,0),this.plan?.nextBillingDate&&this.formatDateToYYYYMMDD(new Date(this.plan.nextBillingDate))===this.formatDateToYYYYMMDD(r))return{valid:!1,errorCode:`same_next_billing_date`};if(r<n)return{valid:!1,errorCode:`date_too_early`};let i=new Date(n);if(i.setFullYear(i.getFullYear()+1),r>i)return{valid:!1,errorCode:`date_out_of_range`};let a=this.plan?.payment&&this.plan.payment.lastBillingDate?.date?new Date(this.plan.payment.lastBillingDate.date):null;if(a){let e=a.getMonth(),n=a.getFullYear(),r=t.getFullYear(),i=t.getMonth();if(r===n&&i===e)return{valid:!0,errorCode:`second_donation_this_month`}}return{valid:!0,errorCode:``}}get minDate(){let e=new Date,t=new Date(e);return t.setDate(e.getDate()+1),`${t.getFullYear()}-${String(t.getMonth()+1).padStart(2,`0`)}-${String(t.getDate()).padStart(2,`0`)}`}get maxDate(){let e=new Date;return`${e.getFullYear()+1}-${String(e.getMonth()+1).padStart(2,`0`)}-${String(e.getDate()).padStart(2,`0`)}`}get editDateForm(){let e=this.plan?.nextBillingDate?this.formatDateToYYYYMMDD(new Date(this.plan.nextBillingDate)):``,t=this.dateInput?.value??e;return a`
      <section>
        <form id="edit-date">
          <p id="form-info-last-donation-date">
            Last donation date: ${this.plan?.lastBillingDateLocale}
          </p>

          <p id="form-info-next-donation-date">
            Next donation date: ${this.plan?.nextBillingDateLocale}
          </p>
          <div class="date-holder">
            <input
              required
              type="date"
              id="edit-date"
              name="edit-date"
              min=${this.minDate}
              max=${this.maxDate}
              .value=${t}
              @focus=${()=>this.clearStatusMessaging()}
              @change=${async()=>{this.clearStatusMessaging(),await this.updateComplete;let e=this.dateInput.value,{valid:t,errorCode:n}=this.validateChosenDate(e);if(this.allowEditing=t,n)if(n===`second_donation_this_month`)this.errorMessage=I[n],this.warningMessage=`You have already made a donation this month.`;else{this.errorMessage=I[n],this.newDate=void 0;return}let r=new Date(`${e}T00:00:00`),i=new Date(r).toISOString();this.newDate=i}}
            />
          </div>
          <div>
            <div class="cta-container">
              <ia-mgc-button
                class="ia-button secondary"
                .clickHandler=${e=>{e.preventDefault(),this.closeForm()}}
              >
                Cancel
              </ia-mgc-button>
              <ia-mgc-button
                id="edit-date"
                class="ia-button primary"
                type="submit"
                .isDisabled=${!this.allowEditing}
                .clickHandler=${async(e,t)=>{if(this.clearStatusMessaging(),!this.newDate){this.errorMessage=`Please enter a valid date`;return}t.isDisabled=!0,await t.updateComplete,this.requestDateUpdate(e)}}
              >
                Update
              </ia-mgc-button>
              <ia-mgc-update-status .status=${this.updateStatus}
                >${this.updateMessage}</ia-mgc-update-status
              >
            </div>
          </div>
          <p class="error error-msg">
            ${this.warningMessage?a`<span>${this.warningMessage}</span><br />`:``}
            ${this.errorMessage}
          </p>
        </form>
      </section>
    `}};L.styles=e`
    input[name='edit-date'] {
      margin: 10px 0;
      min-width: 140px;
      font-family: inherit;
    }

    .cta-container {
      display: flex;
      flex-direction: row;
      flex-wrap: wrap;
      align-content: center;
      justify-content: flex-start;
      align-items: center;
      gap: 10px;
    }

    p.error {
      color: var(--mgc-warning-color-dark, #bb0505);
    }

    ia-mgc-update-status {
      display: inline-block;
      margin-left: 1rem;
    }

    .checkbox-option-container {
      margin: 10px 0;
      display: flex;
    }
  `,d([t({type:Object})],L.prototype,`plan`,void 0),d([t({type:String})],L.prototype,`newDate`,void 0),d([t({type:Boolean,reflect:!0})],L.prototype,`currentlyEditing`,void 0),d([t({type:Boolean})],L.prototype,`allowEditing`,void 0),d([t({type:String})],L.prototype,`updateMessage`,void 0),d([t({type:String})],L.prototype,`errorMessage`,void 0),d([t({type:String})],L.prototype,`warningMessage`,void 0),d([t({type:String})],L.prototype,`updateStatus`,void 0),d([u(`form`)],L.prototype,`form`,void 0),d([u(`form ia-mgc-button#edit-date`)],L.prototype,`formSubmitButton`,void 0),d([u(`form input[name="edit-date"]`)],L.prototype,`dateInput`,void 0),L=d([l(`ia-mgc-edit-date`)],L);var R=class extends c{constructor(){super(...arguments),this.patronWantsToKeepPlan=!0,this.initialCancelRequest=!1}updated(e){e.has(`plan`)&&console.log(`plan updated`,this.plan)}async cancelThisPlan(e){e.preventDefault(),this.patronWantsToKeepPlan=!1,this.dispatchEvent(new Event(`cancelPlan`))}get formId(){return`cancel-donation-form-${this.plan?.id}`}render(){return this.initialCancelRequest?this.confirmCancelation:a`
      <donation-form-section
        badgemode="hidebadge"
        headline="Cancel recurring donation (requires confirmation)"
      >
        <div class="warning">
          <p>
            You can also pause your recurring donation by setting the next
            donation date up to 12 months in the future.
          </p>
          <ia-mgc-button
            class="link cancel"
            .clickHandler=${(e,t)=>{if(t.isDisabled=!0,this.initialCancelRequest){this.initialCancelRequest=!1,this.patronWantsToKeepPlan=!0;return}this.initialCancelRequest=!0}}
            >Let's cancel my donation</ia-mgc-button
          >
        </div>
      </donation-form-section>

      ${this.initialCancelRequest?this.confirmCancelation:n}
    `}get confirmCancelation(){return a`
    <section class="cancel-donation">
    <donation-form-section badgemode="hidebadge" headline="Cancel recurring donation">

      <ia-mgc-button class='text exit-cancel'  @click=${()=>{this.initialCancelRequest=!1,this.patronWantsToKeepPlan=!0}}>X</ia-mgc-button>

      <p>Canceling ends your monthly recurring donation to the Internet Archive, effective immediately. You will not be charged moving forward.</p>
      <p>Canceling does not affect your account or access to the Internet Archive, although you will no longer have access to any of the Monthly Giving Circle perks.</p>
      <p>If you have any questions regarding donations, contact us at <a href="mailto:donations@archive.org">donations@archive.org</a></p>

      <form id=${this.formId} @submit=${e=>this.cancelThisPlan(e)}>
        <div class="checkbox-option-container">
          <input
            id=${`confirm-${this.formId}`}
            type="checkbox"
            required
            @change=${async e=>{e.preventDefault(),this.patronWantsToKeepPlan=!e.target.checked,await this.updateComplete}}>
          <label for=${`confirm-${this.formId}`}><b>I'm sure I want to cancel my recurring donation.</b></label>
        </div>

        <ia-mgc-button
          class="cancel"
          .isDisabled=${this.patronWantsToKeepPlan}
          id=${`submit-${this.formId}`}
          type="submit"
          .clickHandler=${(e,t)=>{t.isDisabled=!0,this.cancelThisPlan(e)}}
        >Cancel recurring donation</ia-mgc-button>
      </form>
    </section>
    <p class="pause-note">
      You can also pause your recurring donation by setting the next donation
      date up to 12 months in the future.
    </p>
    `}};R.styles=e`
    :host {
      --formSectionTitleFontSize: 1.4rem;
    }
    .warning > * {
      margin: 5px 0;
    }

    .cancel-donation {
      display: block;
      border: 2px solid #d9534f;
      background-color: #ffeeee;
    }

    .pause-note {
      font-style: italic;
      margin: 5px 0;
    }

    .cancel-donation > * {
      padding: 5px;
      position: relative;
    }

    ia-mgc-button.exit-cancel {
      --button-border: 1px solid;
      --button-border-radius: 50%;
      position: absolute;
      top: -5px;
      right: -10px;
    }

    ia-mgc-button.link.cancel {
      --link-cancel-color: var(--mgc-warning-color-dark, #bb0505);
    }

    /* keep the red cancel fill while disabled; the button's 0.5 opacity
       fades it to pink until the confirm checkbox is ticked */
    ia-mgc-button.cancel:not(.link) {
      --primaryDisableCTAFill: #d9534f;
      --secondaryCTABorder: #f18286;
    }

    ia-mgc-button {
      --button-height: auto;
    }

    ia-mgc-button > * {
      text-align: left;
      text-wrap: wrap;
    }

    h3 {
      position: relative;
    }

    .checkbox-option-container {
      display: flex;
      margin: 10px 0;
    }
  `,d([t({type:Object})],R.prototype,`plan`,void 0),d([t({type:Boolean,reflect:!0})],R.prototype,`patronWantsToKeepPlan`,void 0),d([t({type:Boolean,reflect:!0})],R.prototype,`initialCancelRequest`,void 0),d([u(`form`)],R.prototype,`form`,void 0),R=d([l(`ia-mgc-cancel-plan`)],R);var z=class extends c{constructor(){super(...arguments),this.displayCreditCard=!1,this.patronEmail=``,this.elementConnected=!1}get creditCardFieldsElement(){return this.closest(`form`)?.querySelector(`credit-card-fields`)??null}createRenderRoot(){return this}disconnectedCallback(){this.elementConnected=!1}connectedCallback(){super.connectedCallback(),console.log(`connectedCallback`,{paymentConfig:this.paymentConfig}),this.elementConnected=!0}updated(e){if(e.has(`elementConnected`)&&this.elementConnected&&this.paymentConfig){let{braintreeAuthToken:e,endpointManager:t,paymentClients:n,environment:r}=this.paymentConfig||{};!this.braintreeManager&&e&&t&&n&&r&&this.setupBraintreeManager()}this.braintreeManager&&e.has(`displayCreditCard`)&&this.displayCreditCard&&this.setupCreditCardHandler()}async validateCreditCardFields(){let e=await this.braintreeManager?.paymentProviders.creditCardHandler.get(),t=!1,n=null;try{n=await e?.tokenizeHostedFields(),t=!0}catch(t){let n=t;switch(e?.showErrorMessage(),n.code){case`HOSTED_FIELDS_FIELDS_EMPTY`:e?.markFieldErrors([_.Number,_.CVV,_.ExpirationDate]);break;case`HOSTED_FIELDS_FIELDS_INVALID`:break;case`HOSTED_FIELDS_TOKENIZATION_FAIL_ON_DUPLICATE`:break;case`HOSTED_FIELDS_TOKENIZATION_CVV_VERIFICATION_FAILED`:e?.markFieldErrors([_.CVV]);break;case`HOSTED_FIELDS_FAILED_TOKENIZATION`:break;case`HOSTED_FIELDS_TOKENIZATION_NETWORK_ERROR`:break;default:break}}return t?n:!1}async setupCreditCardHandler(){let e=await this.braintreeManager?.paymentProviders.creditCardHandler.get();try{await e?.tokenizeHostedFields()}catch{}e?.removeFieldErrors([_.Number,_.CVV,_.ExpirationDate])}render(){return a``}async renderPayPalVaultButton(){console.log(`[PayPal] renderPayPalVaultButton called`);let e=await this.braintreeManager?.paymentProviders.paypalHandler.get();if(console.log(`[PayPal] handler:`,e),!e)return;let t=document.querySelector(`#ia-mgc-paypal-button`);console.log(`[PayPal] container element:`,t);let n=new h({donationType:y.Monthly,amount:this.plan?.amount??0,coverFees:!1}),r=await e.renderPayPalButton({selector:`#ia-mgc-paypal-button`,style:{color:`blue`,shape:`rect`,size:`medium`},donationInfo:n});console.log(`[PayPal] dataSource:`,r),r&&(r.delegate={payPalPaymentStarted:async()=>{console.log(`PayPal payment started`)},payPalPaymentAuthorized:async(e,t)=>{this.handlePayPalAuthorized(t)},payPalPaymentConfirmed:async(e,t)=>{this.handlePayPalAuthorized(t)},payPalPaymentCancelled:async()=>{console.log(`PayPal payment cancelled`)},payPalPaymentError:async(e,t)=>{console.error(`PayPal vault error:`,t),this.dispatchEvent(new CustomEvent(`PayPalVaultError`,{detail:{error:t}}))}})}handlePayPalAuthorized(e){this.dispatchEvent(new CustomEvent(`PayPalVaultAuthorized`,{detail:{paymentMethodInfo:{description:`PayPal - ${e.details.email}`,nonce:e.nonce,type:e.type,details:{email:e.details.email}}}}))}async startVenmoPayment(){var e,t,n;let r=await this.braintreeManager?.paymentProviders.venmoHandler.get();if(r){this.plan?.id&&((e=this.venmoPendingStorage)==null||e.setPending(this.plan.id));try{let e=await r.startPayment();this.plan?.id&&((t=this.venmoPendingStorage)==null||t.clearPending(this.plan.id)),this.dispatchEvent(new CustomEvent(`VenmoAuthorized`,{detail:{paymentMethodInfo:{description:`Venmo - ${e.details.username}`,nonce:e.nonce,type:e.type,details:{username:e.details.username}}}}))}catch(e){this.plan?.id&&((n=this.venmoPendingStorage)==null||n.clearPending(this.plan.id));let t=e?.code;t===`VENMO_APP_CANCELED`||t===`VENMO_CANCELED`?console.log(`Venmo payment cancelled`):(console.error(`Venmo payment error:`,e),this.dispatchEvent(new CustomEvent(`VenmoError`,{detail:{error:e}})))}}}async startGooglePayPayment(){let e=await this.braintreeManager?.paymentProviders.googlePayHandler.get();if(e)try{let t=await e.instance.get();if(!t)return;let n=await t.createPaymentDataRequest({emailRequired:!0,transactionInfo:{currencyCode:`USD`,totalPriceStatus:`FINAL`,totalPrice:`${this.plan?.amount??0}`}}),r=await e.paymentsClient.loadPaymentData(n),i=await t.parseResponse(r);this.dispatchEvent(new CustomEvent(`GooglePayVaultAuthorized`,{detail:{paymentMethodInfo:{description:`Google Pay - ${i.details.cardType} - ${i.details.lastFour}`,nonce:i.nonce,type:i.type,details:{cardType:i.details.cardType,lastFour:i.details.lastFour}}}}))}catch(e){e?.statusCode===`CANCELED`?console.log(`Google Pay payment cancelled`):(console.error(`Google Pay payment error:`,e),this.dispatchEvent(new CustomEvent(`GooglePayError`,{detail:{error:e}})))}}async startApplePayPayment(e){let t=await this.braintreeManager?.paymentProviders.applePayHandler.get();if(t)try{let e=await t.instance.get();if(!e)return;let n=e.createPaymentRequest({total:{label:`Internet Archive Monthly`,amount:`${this.plan?.amount??0}`},requiredBillingContactFields:[`postalAddress`]}),r=new ApplePaySession(3,n);r.onvalidatemerchant=t=>{e.performValidation({validationURL:t.validationURL,displayName:`Internet Archive`},(e,t)=>{if(e){console.error(`Apple Pay merchant validation error:`,e),r.abort(),this.dispatchEvent(new CustomEvent(`ApplePayError`,{detail:{error:e}}));return}r.completeMerchantValidation(t)})},r.onpaymentauthorized=async t=>{try{let n=await e.tokenize({token:t.payment.token});r.completePayment(ApplePaySession.STATUS_SUCCESS),this.dispatchEvent(new CustomEvent(`ApplePayVaultAuthorized`,{detail:{paymentMethodInfo:{description:`Apple Pay - ${n.details.cardType} - ${n.details.dpanLastTwo}`,nonce:n.nonce,type:n.type,details:{cardType:n.details.cardType,lastTwo:n.details.dpanLastTwo}}}}))}catch(e){r.completePayment(ApplePaySession.STATUS_FAILURE),console.error(`Apple Pay tokenization error:`,e),this.dispatchEvent(new CustomEvent(`ApplePayError`,{detail:{error:e}}))}},r.oncancel=()=>{console.log(`Apple Pay payment cancelled`)},r.begin()}catch(e){console.error(`Apple Pay error:`,e),this.dispatchEvent(new CustomEvent(`ApplePayError`,{detail:{error:e}}))}}async checkVenmoRestoration(){var e,t,n,r;let i=this.plan?.id;if(i&&this.venmoPendingStorage?.getPending(i))try{let r=await this.braintreeManager?.paymentProviders.venmoHandler.get();if(!r){(e=this.venmoPendingStorage)==null||e.clearPending(i);return}let a=await r.instance.get();if(!a){(t=this.venmoPendingStorage)==null||t.clearPending(i);return}if((n=this.venmoPendingStorage)==null||n.clearPending(i),!a.hasTokenizationResult())return;let o=await r.startPayment();this.dispatchEvent(new CustomEvent(`VenmoAuthorized`,{detail:{paymentMethodInfo:{description:`Venmo - ${o.details.username}`,nonce:o.nonce,type:o.type,details:{username:o.details.username}}}}))}catch(e){(r=this.venmoPendingStorage)==null||r.clearPending(i);let t=e?.code;t!==`VENMO_APP_CANCELED`&&t!==`VENMO_CANCELED`&&(console.error(`Venmo restoration error:`,e),this.dispatchEvent(new CustomEvent(`VenmoError`,{detail:{error:e}})))}}async setupBraintreeManager(){this.braintreeManager=new oe({paymentClients:this.paymentConfig?.paymentClients??{},endpointManager:this.paymentConfig?.endpointManager,authorizationToken:this.paymentConfig?.braintreeAuthToken??``,venmoProfileId:this.paymentConfig?.venmoProfileId,googlePayMerchantId:this.paymentConfig?.googlePayMerchantId,hostedFieldConfig:{hostedFieldStyle:{},hostedFieldFieldOptions:{number:{selector:`#braintree-creditcard`},cvv:{selector:`#braintree-cvv`},expirationDate:{selector:`#braintree-expiration`}},hostedFieldContainer:this.creditCardFieldsElement?.hostedFieldContainer},hostingEnvironment:this.paymentConfig?.environment,referrer:window.location.href,loggedInUser:this.patronEmail,origin:window.location.origin}),this.braintreeManager.on(`paymentProvidersHostedFieldsRetry`,e=>{let t=new CustomEvent(`paymentProvidersHostedFieldsRetry`,{detail:{retryNumber:e}});this.dispatchEvent(t)}),this.braintreeManager.on(`paymentProvidersHostedFieldsFailed`,e=>{let t=new CustomEvent(`paymentProvidersHostedFieldsFailed`,{detail:{error:e}});this.dispatchEvent(t)}),!this.paymentConfig?.googlePayMerchantId&&this.braintreeManager&&(this.braintreeManager.paymentProviders.googlePayHandler={get:async()=>null}),await this.checkVenmoRestoration(),this.dispatchEvent(new Event(`BraintreeManagerSetupComplete`))}get contactForm(){return this.querySelector(`form[name="contact-form"]`)}};d([t({type:Object})],z.prototype,`plan`,void 0),d([t({type:Boolean})],z.prototype,`displayCreditCard`,void 0),d([t({type:String})],z.prototype,`patronEmail`,void 0),d([t({type:Object})],z.prototype,`paymentConfig`,void 0),d([t({type:Object})],z.prototype,`braintreeManager`,void 0),d([s()],z.prototype,`elementConnected`,void 0),d([t({type:Object})],z.prototype,`venmoPendingStorage`,void 0),z=d([l(`ia-mgc-braintree-manager`)],z);var ye=`creditCard`,B=class extends c{constructor(){super(...arguments),this.paymentConfig={braintreeAuthToken:``,endpointManager:void 0,paymentClients:void 0,environment:`Development`,venmoProfileId:``,googlePayMerchantId:``,referrer:``,origin:``},this.patronEmail=``,this.currentlyEditing=!1,this.newPaymentMethod=``,this.selectedPaymentProvider=``,this.paymentSelectorFirstUpdated=!1,this.updateMessage=``,this.updateStatus=``}createRenderRoot(){return this}firstUpdated(){this.checkAndRestoreVenmoState()}checkAndRestoreVenmoState(){this.plan?.id&&this.venmoPendingStorage?.getPending(this.plan.id)&&(this.currentlyEditing=!0,this.selectedPaymentProvider=g.Venmo)}submitPaymentMethodChange(e){e.preventDefault()}selectProvider(e){this.selectedPaymentProvider=e,this.authorizedPaymentMethodRequest=void 0}clearStatusMessaging(){this.updateMessage=``,this.updateStatus=``}async paymentMethodUpdated(e){if(this.updateStatus=e,this.updateMessage=e===`success`?`Payment method updated`:`Failed to update date, please try again`,e===`success`){this.currentlyEditing=!1,this.selectedPaymentProvider=``,this.authorizedPaymentMethodRequest=void 0;return}this.updateRequestButton.isDisabled=!1}get updateRequestButton(){return this.querySelector(`ia-mgc-button#edit-plan-payment-method-submit`)}get contactFormElement(){return this.querySelector(`contact-form`)}get contactFormSection(){return a`
      <donation-form-section
        badgemode="hidebadge"
        headline=${this.selectedPaymentProvider===g.Venmo?`Help us stay in touch`:`Enter payment information`}
        id="contactFormSection"
      >
        <div class="credit-card-fields">
          <slot
            style="
              display: block;
              width: 100%;
              border: 1px solid blue;
              height: inherit;
          "
          ></slot>
        </div>
      </donation-form-section>
    `}get paymentFormSection(){return a``}get creditCardElement(){return this.querySelector(`contact-form`)}get creditCardFieldsElement(){return this.querySelector(`credit-card-fields`)}get braintreeManagerElement(){return this.querySelector(`ia-mgc-braintree-manager`)}get paymentMethodDetail(){let{paymentMethodType:e,paypalEmail:t,venmoUsername:n,cardType:r,last4:i}=this.plan?.payment??{};return e===g.PayPal?t??``:e===g.Venmo?n??``:`${r} - ${i}`}render(){let e=this.selectedPaymentProvider===g.CreditCard||this.selectedPaymentProvider===g.Venmo,t=this.selectedPaymentProvider===g.CreditCard,r=!!this.selectedPaymentProvider,i=this.selectedPaymentProvider===g.CreditCard||!!this.authorizedPaymentMethodRequest;return a`
      <style>
        ${this.styles}
      </style>
      <donation-form-section badgemode="hidebadge" headline="Payment method">
        ${this.currentlyEditing?n:a`<ia-mgc-form-section-info
              @editingRequested=${()=>{this.currentlyEditing=!0,this.clearStatusMessaging()}}
              ><span>
                ${this.plan?.payment?.paymentMethodType===ye?`Credit Card`:this.plan?.payment?.paymentMethodType}:
                ${this.paymentMethodDetail}
              </span></ia-mgc-form-section-info
            >`}
        ${this.currentlyEditing?a`
            <form id="edit-plan-payment-method" @submit=${this.submitPaymentMethodChange}>
              <payment-selector
                .paymentProviders=${this.braintreeManager?.paymentProviders}
                @firstUpdated=${e=>{var t;this.paymentSelectorFirstUpdated=!0,(t=e.target)==null||t.showPaypalButton()}}
                @creditCardSelected=${()=>{this.selectProvider(g.CreditCard)}}
                @venmoSelected=${()=>{this.selectProvider(g.Venmo)}}
                @applePaySelected=${e=>{var t;this.selectProvider(g.ApplePay),(t=this.braintreeManagerElement)==null||t.startApplePayPayment(e.detail.originalEvent)}}
                @googlePaySelected=${()=>{var e;this.selectProvider(g.GooglePay),(e=this.braintreeManagerElement)==null||e.startGooglePayPayment()}}
                @paypalSelected=${()=>{this.selectProvider(g.PayPal)}}
                @resetPaymentMethod=${async()=>{this.selectProvider(``)}}
                tabindex="0"
              >
                <div slot="paypal-button">
                  <div id="ia-mgc-paypal-button"></div>
                </div>
                <div
                  slot="credit-card-fields"
                  class="${t?``:`hidden`}"
                >
                  <credit-card-fields></credit-card-fields>
                </div>
              </payment-selector>

              <div class="${e?``:`hidden`}">
                <contact-form .donorEmail=${this.patronEmail}></contact-form>
              </div>

              <ia-mgc-braintree-manager
                class="${r?``:`hidden`}"
                .displayCreditCard=${t}
                .plan=${this.plan}
                .paymentConfig=${this.paymentConfig}
                .venmoPendingStorage=${this.venmoPendingStorage}
                @BraintreeManagerSetupComplete=${()=>{var e;this.braintreeManager=this.braintreeManagerElement?.braintreeManager,(e=this.braintreeManagerElement)==null||e.renderPayPalVaultButton()}}
                @PayPalVaultAuthorized=${e=>{this.handlePayPalVaultAuthorized(e)}}
                @PayPalVaultError=${()=>{this.updateStatus=`fail`,this.updateMessage=`PayPal error, please try again`}}
                @VenmoAuthorized=${e=>{this.handleVenmoAuthorized(e)}}
                @VenmoError=${()=>{this.showVenmoError(`Venmo error, please try again`),this.dispatchEvent(new CustomEvent(`VenmoError`,{bubbles:!0,composed:!0}))}}
                @GooglePayVaultAuthorized=${e=>{this.handleGooglePayVaultAuthorized(e)}}
                @GooglePayError=${()=>{this.updateStatus=`fail`,this.updateMessage=`Google Pay error, please try again`}}
                @ApplePayVaultAuthorized=${e=>{this.handleApplePayVaultAuthorized(e)}}
                @ApplePayError=${()=>{this.updateStatus=`fail`,this.updateMessage=`Apple Pay error, please try again`}}
              ></ia-mgc-braintree-manager>

              <ia-mgc-button
                id="edit-plan-payment-method-cancel"
                class="secondary"
                .clickHandler=${()=>{var e;this.selectedPaymentProvider===g.Venmo&&this.plan?.id&&((e=this.venmoPendingStorage)==null||e.clearPending(this.plan.id)),this.currentlyEditing=!1,this.selectedPaymentProvider=``,this.authorizedPaymentMethodRequest=void 0,this.clearStatusMessaging()}}
                >Cancel</ia-mgc-button
              >
              <ia-mgc-button
                id="edit-plan-payment-method-submit"
                class="primary"
                type="submit"
                .isDisabled=${!i}
                .clickHandler=${async(e,t)=>{let n=t;if(n.isDisabled=!0,this.selectedPaymentProvider!==g.CreditCard){if(!this.authorizedPaymentMethodRequest){n.isDisabled=!1;return}this.dispatchEvent(new CustomEvent(`UpdatePaymentMethod`,{detail:{newPaymentMethodRequest:this.authorizedPaymentMethodRequest}}));return}let r=this.creditCardElement?.reportValidity(),i=await this.braintreeManagerElement?.validateCreditCardFields();if(!r||!i){n.isDisabled=!1;return}let a=new E({paymentMethodInfo:i,donorContactInfo:this.contactFormElement?.donorContactInfo,paymentProvider:g.CreditCard});this.dispatchEvent(new CustomEvent(`UpdatePaymentMethod`,{detail:{newPaymentMethodRequest:a}}))}}
                >Update payment method</ia-mgc-button
              >
              ${this.selectedPaymentProvider===g.Venmo?a`<ia-mgc-button
                      id="edit-plan-payment-method-venmo-submit"
                      class="primary"
                      .clickHandler=${async(e,t)=>{let n=t;if(n.isDisabled=!0,!this.contactFormElement?.reportValidity()){n.isDisabled=!1;return}this.dispatchEvent(new CustomEvent(`VenmoRedirectStarted`,{bubbles:!0,composed:!0})),await this.braintreeManagerElement?.startVenmoPayment(),n.isDisabled=!1}}
                      >Pay with Venmo</ia-mgc-button
                    >`:n}
              <ia-mgc-update-status .status=${this.updateStatus}
                >${this.updateMessage}</ia-mgc-update-status
              ></ia-mgc-form-section-info>
              <div class="secure-process-note">
                ${se} Your payment will be securely processed
              </div>
            </form>
          `:n}
      </donation-form-section>
    `}showVenmoError(e=`Venmo payment cancelled, please try again.`){this.updateStatus=`fail`,this.updateMessage=e}handleVenmoAuthorized(e){let{paymentMethodInfo:t}=e.detail;this.authorizedPaymentMethodRequest=new E({paymentMethodInfo:t,donorContactInfo:this.contactFormElement?.donorContactInfo??{},paymentProvider:g.Venmo})}handleGooglePayVaultAuthorized(e){let{paymentMethodInfo:t}=e.detail;this.authorizedPaymentMethodRequest=new E({paymentMethodInfo:t,donorContactInfo:this.contactFormElement?.donorContactInfo??{},paymentProvider:g.GooglePay})}handleApplePayVaultAuthorized(e){let{paymentMethodInfo:t}=e.detail;this.authorizedPaymentMethodRequest=new E({paymentMethodInfo:t,donorContactInfo:this.contactFormElement?.donorContactInfo??{},paymentProvider:g.ApplePay})}handlePayPalVaultAuthorized(e){let{paymentMethodInfo:t}=e.detail,n=t?.details?.email??``,r=this.contactFormElement?.donorContactInfo??{customer:{email:n}};this.authorizedPaymentMethodRequest=new E({paymentMethodInfo:t,donorContactInfo:r,paymentProvider:g.PayPal})}get styles(){return e`
      payment-selector {
        display: block;
        margin-bottom: 5px;
        padding-bottom: 10px;
      }

      ia-mgc-braintree-manager {
        display: block;
        margin: 10px 0;
      }

      .secure-process-note {
        margin-top: 0.5em;
        font-size: 0.75em;
      }

      .secure-process-note svg {
        width: 1.2rem;
        height: 1.5rem;
        vertical-align: bottom;
      }

      .hidden {
        visibility: hidden;
        height: 1px;
        width: 1px;
      }

      #ia-mgc-paypal-button {
        opacity: 0.001;
        width: var(--paymentButtonWidth, 5rem);
        height: var(--paymentButtonHeight, 3.2rem);
        overflow: hidden;
      }
    `}};d([t({type:Object})],B.prototype,`plan`,void 0),d([t({type:Object})],B.prototype,`paymentConfig`,void 0),d([t({type:String})],B.prototype,`patronEmail`,void 0),d([t({type:Boolean,reflect:!0})],B.prototype,`currentlyEditing`,void 0),d([t({type:String,reflect:!0})],B.prototype,`newPaymentMethod`,void 0),d([t({type:Object})],B.prototype,`braintreeManager`,void 0),d([t({type:String,reflect:!0})],B.prototype,`selectedPaymentProvider`,void 0),d([t({type:Boolean})],B.prototype,`paymentSelectorFirstUpdated`,void 0),d([t({type:String})],B.prototype,`updateMessage`,void 0),d([t({type:String})],B.prototype,`updateStatus`,void 0),d([t({type:Object})],B.prototype,`venmoPendingStorage`,void 0),d([s()],B.prototype,`authorizedPaymentMethodRequest`,void 0),B=d([l(`ia-mgc-edit-payment-method`)],B);var V=class extends c{constructor(){super(...arguments),this.patronEmail=``,this.canEditPaymentMethod=!1,this.paymentConfig={referrer:``,origin:``,braintreeAuthToken:``,venmoProfileId:``,googlePayMerchantId:``,environment:v.Development,paymentClients:void 0,endpointManager:void 0}}createRenderRoot(){return this}paymentMethodUpdates(e){this.querySelector(`ia-mgc-edit-payment-method`).paymentMethodUpdated(e)}amountUpdates(e){this.querySelector(`ia-mgc-edit-plan-amount`).amountUpdated(e)}dateUpdates(e){this.querySelector(`ia-mgc-edit-date`).dateUpdated(e)}render(){return a`
      <section class="mgc-edit-plan">
        <ia-mgc-edit-plan-amount
          .plan=${this.plan}
          @updateAmount=${e=>{var t;let{amount:n,baseAmount:r,coverFees:i,feeCovered:a}=e.detail;this.plan&&((t=this.updateAmountHandler)==null||t.call(this,this.plan,{amount:n,baseAmount:r,coverFees:i,feeCovered:a}))}}
        ></ia-mgc-edit-plan-amount>
        ${this.canEditPaymentMethod?a`
              <hr />
              <ia-mgc-edit-payment-method
                .plan=${this.plan}
                .patronEmail=${this.patronEmail}
                .paymentConfig=${this.paymentConfig}
                .venmoPendingStorage=${this.venmoPendingStorage}
                @UpdatePaymentMethod=${e=>{let{newPaymentMethodRequest:t}=e.detail;this.plan&&this.dispatchEvent(new CustomEvent(`UpdatePaymentMethod`,{detail:{plan:this.plan,newPaymentMethodRequest:t}}))}}
              >
              </ia-mgc-edit-payment-method>
            `:``}
        <hr />
        <ia-mgc-edit-date
          @updateDate=${e=>{let{newDate:t}=e.detail;this.plan&&this.dispatchEvent(new CustomEvent(`updateDate`,{detail:{plan:this.plan,newDate:t}}))}}
          .plan=${this.plan}
        ></ia-mgc-edit-date>
        <hr />
        <ia-mgc-cancel-plan
          .plan=${this.plan}
          @cancelPlan=${()=>{this.dispatchEvent(new Event(`cancelPlan`))}}
        ></ia-mgc-cancel-plan>
        <hr />
        <p class="email-edit-plan">
          Need to update your plan further? Please email us at
          <a href=${this.mailToText()}>donations@archive.org</a>.
        </p>
      </section>
    `}mailToText(){return`mailto:donations@archive.org?subject=I'd like to update my monthly donation`}};d([t({type:Object})],V.prototype,`plan`,void 0),d([t({type:String})],V.prototype,`patronEmail`,void 0),d([t({type:Boolean})],V.prototype,`canEditPaymentMethod`,void 0),d([t({type:Object})],V.prototype,`updateAmountHandler`,void 0),d([t({type:Object})],V.prototype,`venmoPendingStorage`,void 0),d([t({type:Object})],V.prototype,`paymentConfig`,void 0),d([u(`#braintree-creditcard`)],V.prototype,`braintreeNumberInput`,void 0),d([u(`#braintree-expiration`)],V.prototype,`braintreeExpirationDateInput`,void 0),d([u(`#braintree-cvv`)],V.prototype,`braintreeCVVInput`,void 0),d([u(`#braintree-error-message`)],V.prototype,`braintreeErrorMessage`,void 0),V=d([l(`ia-mgc-edit-plan`)],V);var H;(function(e){e.welcome=`ShowWelcome`,e.receipts=`ShowReceipts`,e.plans=`ShowPlans`,e.editPlan=`ShowEditForm`})(H||(H={}));var U=class extends c{constructor(){super(...arguments),this.patronName=``,this.patronEmail=``,this.canEditPaymentMethod=!1,this.receipts=[],this.updates=[],this.plans=[],this.viewToDisplay=`welcome`,this.canEdit=!0,this.venmoPendingStorage=new ge,this.paymentConfig={referrer:``,origin:``,braintreeAuthToken:``,venmoProfileId:``,googlePayMerchantId:``,environment:v.Development,paymentClients:void 0,endpointManager:{}}}createRenderRoot(){return this}updated(e){e.has(`plans`)&&(this.viewToDisplay=this.plans.length?`plans`:`welcome`)}get receiptListElement(){return this.querySelector(`ia-mgc-receipts`)}get editFormElement(){return this.querySelector(`ia-mgc-edit-plan`)}updateReceived(e){this.updates.unshift(e);let{plan:t,donationId:n=``}=e,r=t?.id??n;if(e.action===`paymentMethodUpdate`){this.editFormElement.paymentMethodUpdates(e.status);return}if(e.action===`amountUpdate`){this.editFormElement.amountUpdates(e.status);return}if(e.action===`dateUpdate`){this.editingThisPlan=t,this.editFormElement.dateUpdates(e.status);return}if(e.action===`cancel`||t?.hasBeenCancelled){this.editingThisPlan=void 0,this.viewToDisplay=`plans`;return}this.receiptListElement.emailSent({id:r,emailStatus:e.status})}render(){let e=this.viewToDisplay===`editPlan`&&this.editingThisPlan;return a`
      ${this.sectionTitle}
      ${e?a`<ia-mgc-edit-plan
            .canEditPaymentMethod=${this.canEditPaymentMethod}
            .patronEmail=${this.patronEmail}
            .plan=${this.editingThisPlan}
            .paymentConfig=${this.paymentConfig}
            .venmoPendingStorage=${this.venmoPendingStorage}
            @cancelPlan=${()=>{this.dispatchEvent(new CustomEvent(`cancelPlan`,{detail:{plan:this.editingThisPlan}}))}}
            .updateAmountHandler=${(e,t)=>{console.log(`updateAmount`,e,{...t}),this.dispatchEvent(new CustomEvent(`updateAmount`,{detail:{plan:e,amountOptions:t}}))}}
            @updateDate=${e=>{let{newDate:t}=e.detail;console.log(`updateDate`,t),this.dispatchEvent(new CustomEvent(`updateDate`,{detail:{plan:this.editingThisPlan,newDate:t}}))}}
            @UpdatePaymentMethod=${e=>{let{newPaymentMethodRequest:t}=e.detail;console.log(`UpdatePaymentMethod`,t),console.warn(`UPDATE PAYMENT METHOD`,{plan:this.editingThisPlan,newPaymentMethodRequest:t}),this.dispatchEvent(new CustomEvent(`UpdatePaymentMethod`,{detail:{plan:this.editingThisPlan,newPaymentMethodRequest:t}}))}}
          >
          </ia-mgc-edit-plan>`:this.nonEditView}
    `}get sectionTitle(){let e=``,t=``,n=a``,r=a`
      <ia-mgc-button
        class="link slim"
        .clickHandler=${async()=>{this.viewToDisplay=`receipts`,await this.updateComplete,this.dispatchEvent(new Event(H.receipts))}}
      >
        View recent donation history
      </ia-mgc-button>
    `,i=this.receipts.length&&(this.viewToDisplay===`plans`||this.viewToDisplay===`welcome`);switch(this.viewToDisplay){case`receipts`:e=`Recent donations`,t=`default`,n=a`<ia-mgc-button
          class="primary"
          id="close-receipts"
          .clickHandler=${async()=>{this.viewToDisplay=this.plans.length?`plans`:`welcome`;let e=this.plans.length?H.plans:H.welcome;this.dispatchEvent(new Event(e)),this.updates=[],await this.updateComplete}}
        >
          Back to account settings
        </ia-mgc-button>`;break;case`editPlan`:e=`Monthly Giving Circle`,t=`default`,n=a`<ia-mgc-button
          class="primary"
          id="close-edit-plan"
          .clickHandler=${async()=>{this.viewToDisplay=this.plans.length?`plans`:`welcome`;let e=this.plans.length?H.plans:H.welcome;this.dispatchEvent(new Event(e)),this.updates=[],await this.updateComplete}}
        >
          Back to account settings
        </ia-mgc-button>`;break;default:e=`Monthly Giving Circle`,t=`heart`,i&&(n=r);break}return a`
      <ia-mgc-title titleStyle=${t}>
        <span slot="title">${e}</span>
        <span slot="action">${n}</span>
      </ia-mgc-title>
    `}get nonEditView(){return this.viewToDisplay===`receipts`?a`
        <ia-mgc-receipts
          .receipts=${this.receipts}
          @EmailReceiptRequest=${e=>{console.log(`EmailReceiptRequest`,e.detail),this.dispatchEvent(new CustomEvent(`EmailReceiptRequest`,{detail:{...e.detail}}))}}
        ></ia-mgc-receipts>
      `:a`
      ${this.viewToDisplay===`plans`&&this.plans.length?a`
            <ia-mgc-plans
              @editThisPlan=${async e=>{this.editingThisPlan=e.detail.plan,this.viewToDisplay=`editPlan`,this.dispatchEvent(new Event(H.editPlan)),await this.updateComplete}}
              .plans=${this.plans}
              .canEdit=${this.canEdit}
            ></ia-mgc-plans>
          `:a`<ia-mgc-welcome
            .patronName=${this.patronName}
          ></ia-mgc-welcome>`}
    `}};d([t({type:String})],U.prototype,`patronName`,void 0),d([t({type:String})],U.prototype,`patronEmail`,void 0),d([t({type:Boolean})],U.prototype,`canEditPaymentMethod`,void 0),d([t({type:Array})],U.prototype,`receipts`,void 0),d([t({type:Array})],U.prototype,`updates`,void 0),d([t({type:Array})],U.prototype,`plans`,void 0),d([t({type:Object})],U.prototype,`editingThisPlan`,void 0),d([t({type:String,reflect:!0})],U.prototype,`viewToDisplay`,void 0),d([t({type:Boolean,reflect:!0})],U.prototype,`canEdit`,void 0),d([t({type:Object})],U.prototype,`venmoPendingStorage`,void 0),d([t({type:Object})],U.prototype,`paymentConfig`,void 0),U=d([l(`ia-monthly-giving-circle`)],U);var be=class{constructor(e){this.receipt=e}get amount(){return M(this.receipt.total_amount,this.receipt.currency??`USD`)}get isTest(){return this.receipt.isTest??!1}get id(){return this.receipt.token??`no token found`}get date(){return this.receipt.date??`no date found`}},xe=class{constructor(e){this.plan=e,this.currency=e.currency??`USD`}get id(){return this.plan.token}get amount(){return this.plan.amount}get amountFormatted(){return M(this.plan.amount,this.currency)}get payment(){return this.plan.btdata}setAmount(e){this.plan.oldAmount=this.plan.amount,this.plan.amount=e}get nextBillingDate(){return this.payment?.nextBillingDate?.date??``}setNextBillingDate(e){this.payment&&(this.payment.nextBillingDate.oldDate=this.payment.nextBillingDate.date,this.payment.nextBillingDate.date=e)}formatDateUTC(e){let t=new Date(e);return Number.isNaN(t.getTime())?`Invalid date`:new Intl.DateTimeFormat(`en-US`,{year:`numeric`,month:`short`,day:`numeric`,timeZone:`UTC`}).format(t)}get nextBillingDateLocale(){let e=this.payment?.nextBillingDate?.date??``;return e?this.formatDateUTC(e):`not found`}get lastBillingDateLocale(){let e=this.payment?.lastBillingDate?.date??``;return e?this.formatDateUTC(e):``}get hasBeenCancelled(){return this.plan.isCancelled===!0}get isTest(){return this.plan.is_test}cancelPlan(){this.plan.isCancelled=!0}setNewProcessorId(e){let t=this.plan.processor_id;this.plan.processor_id=e,this.plan.oldProcessorId=t}setNewPaymentMethod(e){let t=this.payment,{details:n,type:r}=e.paymentMethodInfo,i=n.email??n.description??`not_found`,a;switch(e.paymentProvider){case g.PayPal:a={...this.plan.btdata,paymentMethodType:`PayPal`,paypalEmail:i,cardType:null,last4:null,expirationMonth:null,expirationYear:null};break;case g.Venmo:a={...this.plan.btdata,paymentMethodType:g.Venmo,venmoUsername:n.username,cardType:null,last4:null,expirationMonth:null,expirationYear:null};break;case g.GooglePay:a={...this.plan.btdata,paymentMethodType:g.GooglePay,cardType:n.cardType??null,last4:n.lastFour??null,expirationMonth:null,expirationYear:null};break;case g.ApplePay:a={...this.plan.btdata,paymentMethodType:g.ApplePay,cardType:n.cardType??null,last4:n.lastTwo??null,expirationMonth:null,expirationYear:null};break;default:a={...this.plan.btdata,...n,paymentMethodType:r,last4:n.lastFour??`unknown`}}this.plan.old_btData=t,this.plan.btdata=a}},Se=class{constructor(e,t=`archive.org`,n){this.MGC_SERVICE_PATH=`/services/donations/monthly-manage.php`,this.primaryToken=e,this.serviceUrl=this.getServiceUrl(t),this.fetchHandler=n}async fetchPlans(){let e=this.buildParams(`getAccounts`,this.primaryToken);try{let t=await this.makeRequest(e);if(!t.success||!t.value)throw Error(`Failed to fetch plans`);return(Object.values(t.value)??[]).toReversed().map(e=>new xe(e))}catch(e){throw Error(`Error fetching plans: ${e instanceof Error?e.message:String(e)}`)}}async fetchReceipts(){let e=this.buildParams(`getContributions`,this.primaryToken);try{let t=await this.makeRequest(e);if(!t.success||!t.value)throw Error(`Failed to fetch receipts`);return(Object.values(t.value)??[]).toReversed().map(e=>{let t=e,n=t.token,r=new Date(t.receive_date??``).toLocaleString(`EN-US`,{year:`numeric`,month:`short`,day:`numeric`});return new be({...e,date:r,id:n})})}catch(e){throw Error(`Error fetching receipts: ${e instanceof Error?e.message:String(e)}`)}}async sendReceipt(e){return this.makeRequest(this.buildParams(`sendReceipt`,e.receipt.token))}async cancelPlan(e){return this.makeRequest(this.buildParams(`cancelSubscription`,e.plan.token))}async updateDate(e,t){console.log(`***** UPDATE DATE SERVICE ****`,e);let n=this.buildParams(`updateSubscriptionDate`,e.plan.token);return n.append(`date`,t),n.append(`processor_id`,e.plan.processor_id??``),this.makeRequest(n)}async updateAmount({token:e,amount:t,baseAmount:n,feeCovered:r,coverFees:i}){if(!e)throw Error(`Token is required to update the amount`);let a=this.buildParams(`updateAmount`,e);return a.append(`amount`,t.toString()),a.append(`base_amount`,(n??0).toString()),a.append(`fee_amount_covered`,(r??0).toString()),a.append(`fee_covered`,(i??!1).toString()),this.makeRequest(a)}async updatePaymentMethod(e,t){let{plan:n,donorContactInfo:r}=e,{billing:i,customer:a}=r??{},o=n.plan.token,s=this.buildParams(`updatePaymentMethod`,o);return s.append(`nonce`,t),s.append(`streetAddress`,i?.streetAddress??``),s.append(`extendedAddress`,i?.extendedAddress??``),s.append(`locality`,i?.locality??``),s.append(`region`,i?.region??``),s.append(`postalCode`,i?.postalCode??``),s.append(`countryCode`,i?.countryCodeAlpha2??``),s.append(`firstName`,a?.firstName??``),s.append(`lastName`,a?.lastName??``),s.append(`email`,a?.email??``),m(`Updating payment method with params:`,s.toString()),this.makeRequest(s)}async makeRequest(e){try{let t=await(await this.fetchHandler).fetchApiResponse(this.serviceUrl,{method:`POST`,includeCredentials:!0,headers:{"Content-Type":`application/x-www-form-urlencoded`},body:e});return m(`Response from MonthlyGivingService:`,e,t),t}catch(e){throw Error(`Error making request: ${e instanceof Error?e.message:String(e)}`)}}buildParams(e,t){let n=new URLSearchParams;return n.append(`op`,e),n.append(`token`,t),n}getServiceUrl(e){return new URL(this.MGC_SERVICE_PATH,`https://${e}`).toString()}},W=class extends c{constructor(...e){super(...e),this.mgcToken=``,this.patronEmail=``,this.mgcViewName=`welcome`,this.canEditPaymentMethod=!1,this.receipts=[],this.plans=[]}willUpdate(e){e.has(`appServices`)&&this.appServices&&this.setupAppServicesDependents(),e.has(`mgcToken`)&&this.mgcToken&&this.fetchMGCInfo()}createRenderRoot(){return this}async fetchMGCInfo(){let e=await this.archiveApiUrl,t=(await this.appServices?.get())?.fetchHandler.get();this.monthlyGivingService=new Se(this.mgcToken,e,t),await this.fetchPlans(),await this.fetchReceipts()}async setupPaymentConfig(){let e=await this.appServices?.get(),t=await e?.lazyLoaderService.get(),n=await e?.appConfig.get(),r=n?.environment??`prod`,i=n?.documentReferrer??``,a=n?.currentBaseUrl??``,{braintreeAuthToken:o,venmoProfileId:s,googlepayMerchantId:c}=n?.donationForm??{};return{braintreeAuthToken:o??``,venmoProfileId:s??``,googlePayMerchantId:c??``,environment:r,referrer:i,origin:a,paymentClients:new ae(t,r),endpointManager:{submitData:async e=>{m(`PaymentConfig endpointManager.submitData called with:`,e)},donationSuccessful:e=>{m(`PaymentConfig endpointManager.donationSuccessful called with:`,e)}}}}get braintreeAuthToken(){return this.appServices?.get().then(e=>e?.appConfig.get().then(e=>e?.donationForm.braintreeAuthToken??``))??Promise.resolve(``)}get archiveApiUrl(){return this.appServices?.get().then(e=>e?.appConfig.get().then(e=>e?.archiveApiUrl))??Promise.resolve(`archive.org`)}async setupAppServicesDependents(){let e=await this.appServices?.get();await Promise.all([this.modalManager=await e?.modalManager.get(),this.analyticsManager=await e?.analyticsHandler.get(),this.paymentConfig=await this.setupPaymentConfig()])}render(){return m(`Rendering Monthly Giving Circle Controller with state:`,{plans:this.plans,receipts:this.receipts,mgcViewName:this.mgcViewName,canEditPaymentMethod:this.canEditPaymentMethod,paymentConfig:this.paymentConfig,patronEmail:this.patronEmail}),a`
      <ia-monthly-giving-circle
        .patronEmail=${this.patronEmail}
        .plans=${this.plans}
        .receipts=${this.receipts}
        .viewToDisplay=${this.mgcViewName}
        ?canEditPaymentMethod=${this.canEditPaymentMethod}
        .paymentConfig=${this.paymentConfig}
        @ShowWelcome=${()=>{this.mgcViewName=`welcome`,this.emitViewToDisplayChanged()}}
        @ShowReceipts=${async()=>{this.trackEvent(`ShowReceipts`),this.mgcViewName=`receipts`,this.emitViewToDisplayChanged()}}
        @ShowPlans=${()=>{this.trackEvent(`ShowPlans`),this.mgcViewName=`plans`,this.emitViewToDisplayChanged()}}
        @ShowEditForm=${()=>{this.trackEvent(`ShowEditForm`),this.mgcViewName=`editPlan`,this.emitViewToDisplayChanged()}}
        @EmailReceiptRequest=${e=>{let t=e.detail;this.sendReceipt(t.donation)}}
        @cancelPlan=${e=>{let t=e.detail;this.cancelThisPlan(t.plan)}}
        @updateAmount=${e=>{let{plan:t,amountOptions:n}=e.detail;this.editAmount(t,n)}}
        @updateDate=${e=>{this.trackEvent(`UpdateDateRequested`);let{plan:t,newDate:n}=e.detail;this.editDate(t,n)}}
        @PayPalVaultError=${()=>this.handlePayPalVaultError()}
        @VenmoRedirectStarted=${()=>this.displayVenmoRedirectModal()}
        @VenmoError=${()=>this.modalManager?.closeModal()}
        @UpdatePaymentMethod=${e=>{this.trackEvent(`UpdatePaymentMethodRequested`);let{newPaymentMethodRequest:t,plan:n}=e.detail,{paymentMethodInfo:r,donorContactInfo:i,paymentProvider:a}=t||{};m(`NEW PAYMENT METHOD REQUEST:`,t),this.editPaymentMethod({plan:n,donorContactInfo:i,paymentMethodInfo:r,paymentProvider:a},r.nonce)}}
      ></ia-monthly-giving-circle>
    `}async fetchPlans(){try{this.plans=await this.monthlyGivingService?.fetchPlans()??[],this.trackEvent(`HasActivePlans`)}catch{this.trackEvent(`ErrorPlansFetch`)}}async fetchReceipts(){try{this.receipts=await this.monthlyGivingService?.fetchReceipts()??[],this.trackEvent(`HasReceipts`)}catch{this.trackEvent(`ErrorFetchReceipts`)}}async sendReceipt(e){try{let t=await this.monthlyGivingService?.sendReceipt(e);if(t?.success&&t.value)await this.handleUpdateState(`receiptSent`,`success`,e);else throw Error(`Failed to do action: Email Receipt`)}catch(t){await this.handleUpdateState(`receiptSent`,`fail`,e,String(t))}}async cancelThisPlan(e){if(e.hasBeenCancelled){await this.handleUpdateState(`cancel`,`success`,e);return}this.displayProcessingModal();try{let t=await this.monthlyGivingService?.cancelPlan(e);if(t?.success&&t.value){let t=this.plans.map(t=>(t.id===e.id&&t.cancelPlan(),t));this.plans=t,await this.handleUpdateState(`cancel`,`success`,e)}else throw Error(`Failed to do action: Cancel Plan`)}catch(t){await this.handleUpdateState(`cancel`,`fail`,e,String(t))}}applyUpdatedAccount(e,t){let[n]=Object.values(t??{});if(!n)throw Error(`Update succeeded but returned no account`);e.plan.token=n.token,e.plan.btdata={cardType:null,last4:null,expirationMonth:null,expirationYear:null,...n.btdata}}async editDate(e,t){this.trackEvent(`EditDateRequested`),this.displayProcessingModal(),e.plan.processor_id;try{let n=await this.monthlyGivingService?.updateDate(e,t);if(n?.success&&n.value){let{new_processor_id:r}=n.value;e.setNextBillingDate(t),e.setNewProcessorId(r),this.applyUpdatedAccount(e,n.value.accounts),await this.handleUpdateState(`dateUpdate`,`success`,e)}else throw Error(`Date update failed: ${n?.error||`Unknown error`}`)}catch(t){let n=t instanceof Error?t.message:String(t);await this.handleUpdateState(`dateUpdate`,`fail`,e,n)}}async editAmount(e,t){this.displayProcessingModal();try{let{amount:n,baseAmount:r,feeCovered:i,coverFees:a}=t,o=await this.monthlyGivingService?.updateAmount({token:e.plan.token,amount:n,baseAmount:r,feeCovered:i,coverFees:a});if(o?.success&&o.value)e.setAmount(t.amount),this.applyUpdatedAccount(e,o.value),await this.handleUpdateState(`amountUpdate`,`success`,e);else throw Error(`Failed to do action: Edit Amount`)}catch(t){await this.handleUpdateState(`amountUpdate`,`fail`,e,String(t))}}async handleUpdateState(e,t,n,r=``){let i={receiptSent:{successEvent:`ReceiptSent`,errorEvent:`ErrorReceiptNotSent`,errorMessage:`Failed to send receipt.`,successMessage:`Receipt Sent.`},cancel:{successEvent:`PlanCancelled`,errorEvent:`ErrorPlanNotCancelled`,errorMessage:`Failed to cancel your plan.`,successMessage:`Plan cancelled.`},amountUpdate:{successEvent:`AmountUpdated`,errorEvent:`ErrorAmountNotUpdated`,errorMessage:`Failed to update amount.`,successMessage:`Amount Updated.`},dateUpdate:{successEvent:`DateUpdated`,errorEvent:`ErrorDateNotUpdated`,errorMessage:`Failed to update date.`,successMessage:`Date Updated.`},paymentMethodUpdate:{successEvent:`PaymentMethodUpdated`,errorEvent:`ErrorPaymentMethodNotUpdated`,errorMessage:`Failed to update payment method.`,successMessage:`Payment Method Updated.`}}[e],a=t===`success`?i.successMessage:r,o={action:e,donationId:n?.id,status:t,message:a,plan:n};this.mgcComponent&&[`receiptSent`,`amountUpdate`,`dateUpdate`,`paymentMethodUpdate`].includes(e)&&this.mgcComponent.updateReceived(o),t===`success`?(this.trackEvent(i.successEvent),this.modalManager?.closeModal(),e===`cancel`&&(this.mgcViewName=`plans`,this.emitViewToDisplayChanged())):(this.trackEvent(i.errorEvent),this.showErrorModal(i.errorMessage,a)),await this.mgcComponent?.updateComplete,m(`Update received & sent to components:`,o)}async editPaymentMethod(e,t){this.displayProcessingModal();let{plan:n,paymentMethodInfo:r,donorContactInfo:i,paymentProvider:a}=e;try{let o=await this.monthlyGivingService?.updatePaymentMethod(e,t);if(o?.success&&o.value){let e=this.plans.find(e=>e.plan.token===n.plan.token);if(!e)throw m(`Token mismatch: no plan with token ${n.plan.token}`),Error(`Failed to update payment method: subscription not found`);e.setNewPaymentMethod(new E({paymentMethodInfo:r,donorContactInfo:i,paymentProvider:a})),this.applyUpdatedAccount(e,o.value),await this.handleUpdateState(`paymentMethodUpdate`,`success`,e)}else throw Error(`Failed to update payment method`)}catch(e){await this.handleUpdateState(`paymentMethodUpdate`,`fail`,n,String(e))}}handlePayPalVaultError(){this.trackEvent(`ErrorPayPalVault`),this.showErrorModal(`PayPal error`,`PayPal authorization failed, please try again.`)}trackEvent(e){this.analyticsManager?.send_event(`MonthlyGivingCircle`,e??`unknownAction`,`AccountSettings`)}emitViewToDisplayChanged(){this.dispatchEvent(new CustomEvent(`viewChanged`,{detail:this.mgcViewName,bubbles:!0,composed:!0}))}async displayVenmoRedirectModal(){let e=new p({title:a`Venmo`,headline:a`Redirecting to Venmo…`,message:a`<p>
        Complete your payment in the Venmo app, then return here.
      </p>`,showProcessingIndicator:!0,processingImageMode:`processing`,showCloseButton:!0,closeOnBackdropClick:!1});this.modalManager?.showModal({config:e,userClosedModalCallback:()=>{(this.mgcComponent?.querySelector(`ia-mgc-edit-payment-method`))?.showVenmoError()}})}async displayProcessingModal(){let e=new p({title:a`Please wait`,headline:a`Processing your request...`,showProcessingIndicator:!0,processingImageMode:`processing`,showCloseButton:!1,closeOnBackdropClick:!0});this.modalManager?.showModal({config:e})}async showErrorModal(e=``,t){let n=new p({title:a`Error`,headline:a`${e}`,message:a`<p>Let's try again.</p>
        <p>${t}</p>`,processingImageMode:`complete`,headerColor:`#d9534f`});this.modalManager?.showModal({config:n})}static get styles(){return e`
      :host {
        display: block;
        font-family: Arial, sans-serif;
      }
      modal-manager[mode='closed'] {
        display: none;
      }
    `}};f([t({type:Object})],W.prototype,`appServices`,void 0),f([t({type:String})],W.prototype,`mgcToken`,void 0),f([t({type:String})],W.prototype,`patronEmail`,void 0),f([t({type:String})],W.prototype,`mgcViewName`,void 0),f([t({type:Boolean,reflect:!0})],W.prototype,`canEditPaymentMethod`,void 0),f([s()],W.prototype,`receipts`,void 0),f([s()],W.prototype,`plans`,void 0),f([s()],W.prototype,`paymentConfig`,void 0),f([u(`ia-monthly-giving-circle`)],W.prototype,`mgcComponent`,void 0),W=f([l(`ia-monthly-giving-circle-controller`)],W);var G=class extends c{constructor(...e){super(...e),this.settingsServicePath=``,this.archiveApiUrl=``,this.identifier=``,this.email=``,this.csrfToken=``,this.thirdPartyAvailable=!0,this.showLoadingIndicator=!1,this.password=``,this.passwordError=``}render(){return a`
      <div class="top-level">
        <h2>${o(`Account settings`)}</h2>
        <p class="subtitle">
          ${o(`To access your account settings, as an extra security measure, you will need to verify your identity.`)}
        </p>
        ${this.authOptionsTemplate}
      </div>
      ${this.iaPasswordFormTemplate}
    `}willUpdate(e){e.has(`appServices`)&&this.appServices&&this.setupAppServices()}async setupAppServices(){let e=await this.appServices?.get();this.fetchHandler=await e?.fetchHandler.get(),this.analyticsHandler=await e?.analyticsHandler.get()}get authOptionsTemplate(){return a`
      <ia-auth-options
        .appServices=${this.appServices}
        .csrfToken=${this.csrfToken}
        .archiveApiUrl=${this.archiveApiUrl}
        .googleClientId=${this.googleClientId}
        .hostPageName=${`verification`}
        .userToVerify=${{identifier:this.identifier,email:this.email}}
        ?showThirdPartyOption=${this.thirdPartyAvailable}
        @otpValidated=${e=>{e.detail.email===this.email&&(this.handleAuthenticationSuccess(e.detail.token??``),this.analyticsHandler?.sendEvent({category:w,action:T.authenticated,label:`otp-authentication-success`}))}}
        @IAThirdPartyAuthVerified=${()=>this.dispatchEvent(new Event(`IAThirdPartyAuthVerified`))}
        @IAThirdPartyAuth:MGCToken=${e=>{this.dispatchEvent(new CustomEvent(`mgcTokenReceived`,{detail:{token:e.detail.token}}))}}
      >
        <span slot="otp-message">${o(`Email me a verification code`)}</span>
      </ia-auth-options>
    `}get iaPasswordFormTemplate(){return a`
      <form
        name="authentication-settings"
        class="ia-authentication"
        autocomplete="off"
        @submit=${this.verifyIAPassword}
      >
        <label for="password">
          ${o(`Enter your Internet Archive password`)}
        </label>
        <div class="password-input">
          <input
            type="password"
            class="form-control password"
            id="password"
            name="password"
            @input=${this.setPassword}
          />
          <ia-password-toggle .passwordInput=${this.passwordField}>
          </ia-password-toggle>
        </div>
        <span class="error-field">${this.passwordError}</span>
        <a
          href="/account/forgot-password?email=${encodeURIComponent(this.email)}"
          class="forgot-pw-link"
        >
          ${o(`Forgot password?`)}
        </a>
        <ia-button
          mode="primary"
          class="submit-btn"
          type="submit"
          ?loading=${this.showLoadingIndicator}
        >
          ${o(`Verify password`)}
        </ia-button>
      </form>
    `}handleAuthenticationSuccess(e){this.dispatchEvent(new CustomEvent(`iaAuthenticated`,{detail:{token:e}}))}handleAuthenticationError(e){this.passwordError=e,this.focusPasswordField(),ne(`IA verification error: ${e}`)}focusPasswordField(){this.passwordField?.focus()}setPassword(e){let t=e.target;this.password=t.value,this.passwordError=``}async verifyIAPassword(e){if(x(e),!this.password){this.focusPasswordField(),this.passwordError=C().PASSWORD_EMPTY;return}let t=!1;try{this.showLoadingIndicator=!0;let e={action:`verify-password`,identifier:this.identifier,password:this.password},n=await this.fetchHandler?.fetchApiPathResponse(this.settingsServicePath,{method:`POST`,body:JSON.stringify(e),includeCredentials:!0,includeCsrfToken:!0,headers:{"Content-Type":`application/json`}});n?.success===!0?(t=!0,this.handleAuthenticationSuccess(n?.value?.token??``)):this.handleAuthenticationError(n?.error??``)}catch(e){this.handleAuthenticationError(`${C().API_ERROR} . ${e}`)}finally{this.analyticsHandler?.sendEvent({category:w,action:T.authenticated,label:t?`password-auth-success`:`password-auth-failure`}),this.showLoadingIndicator=!1}}static get styles(){return[ue,e`
        :host {
          padding-left: calc(25% - 75px);
        }

        h2 {
          margin-bottom: -1.5rem;
        }

        .subtitle {
          margin: 0;
          max-width: 35rem;
          text-align: center;
        }

        .top-level {
          width: calc(75% + 25px);
          display: flex;
          flex-direction: column;
          align-items: center;
        }

        .top-level > * {
          margin-bottom: 2rem;
        }

        .top-level > h2 {
          margin-bottom: 0.5rem;
        }

        .password-input input {
          display: inline-block;
          width: 75%;
          padding: 6px 12px;
          font-size: 14px;
          line-height: 1.42857143;
          background-color: #fdfdfd;
          border: 1px solid #ccc;
          border-radius: 4px;
          box-shadow: inset 0 1px 1px rgba(0, 0, 0, 0.075);
          transition:
            border-color ease-in-out 0.15s,
            box-shadow ease-in-out 0.15s;
        }

        form {
          width: 100%;
          display: flex;
          flex-direction: column;
        }

        form .error-field {
          margin-bottom: 1rem;
          padding: 0;
        }

        label {
          display: block;
          font-weight: bold;
          margin-bottom: 4px;
        }

        .forgot-pw-link {
          display: inherit;
          width: fit-content;
          cursor: pointer;
          text-decoration: none;
          margin-bottom: 2rem;
        }

        .forgot-pw-link:hover {
          text-decoration: underline;
        }

        .submit-btn {
          padding: 0 1rem;
          align-self: center;
          margin-left: calc(-25% + 25px);
        }
      `]}};f([t({type:Object})],G.prototype,`appServices`,void 0),f([t({type:Object,attribute:!1})],G.prototype,`analyticsHandler`,void 0),f([t({type:String})],G.prototype,`settingsServicePath`,void 0),f([t({type:String})],G.prototype,`archiveApiUrl`,void 0),f([t({type:String})],G.prototype,`identifier`,void 0),f([t({type:String})],G.prototype,`email`,void 0),f([t({type:String})],G.prototype,`csrfToken`,void 0),f([t({type:String})],G.prototype,`googleClientId`,void 0),f([t({type:Boolean})],G.prototype,`thirdPartyAvailable`,void 0),f([s()],G.prototype,`fetchHandler`,void 0),f([s()],G.prototype,`showLoadingIndicator`,void 0),f([s()],G.prototype,`password`,void 0),f([s()],G.prototype,`passwordError`,void 0),f([u(`#password`)],G.prototype,`passwordField`,void 0),G=f([l(`ia-verification`),i()],G);var Ce=class extends c{render(){return a` <div class="form-element">
      <h2>Admin functions</h2>
      <hr />
      <ul>
        <li>
          <a href="https://catalogd.archive.org/control/setadmin.php">
            ${o(`Grant or revoke administrator privileges`)}
          </a>
        </li>
        <li>
          <a href="https://catalogd.archive.org/control/useradmin.php">
            ${o(`User Administration`)}
          </a>
        </li>
        <li>
          <a href="https://archive.org/iathreads/forum-new.php">
            ${o(`Make a new Forum`)}
          </a>
        </li>
      </ul>
    </div>`}static get styles(){return e`
      ul {
        list-style: none;
        padding: 0px;
        margin: 0px;
      }
      h2 {
        font-size: 2.1rem;
      }
      ul li {
        padding: 5px 0;
        font-size: 1.4rem;
      }
      ul li a {
        color: var(--ia-theme-link-color);
      }
    `}};Ce=f([l(`admin-links`),i()],Ce);var K=class extends c{constructor(...e){super(...e),this.category=``,this.fieldName=``,this.field={},this.hasDataUpdated=!1,this.showMobileDesc=!1}updated(e){e.has(`field`)&&this.field&&(this.fieldName=this.field.name),this.field.type===`password`&&(this.handlePasswordToggleSetup(),e.has(`hasDataUpdated`)&&this.hasDataUpdated&&this.passwordField&&(this.passwordField.value=``))}handlePasswordToggleSetup(){let e=this.renderRoot.querySelector(`ia-password-toggle`);e&&this.passwordField&&e?.passwordInput!==this.passwordField&&(e.passwordInput=this.passwordField)}handleChange(e,t){let n=e.target,r;r=t.type===`checkbox`&&n instanceof HTMLInputElement?n.checked?`on`:`off`:t.type===`button`?`clicked`:n.value,this.dispatchEvent(new CustomEvent(`setting-changed`,{detail:{category:this.category,key:t.key,value:r},bubbles:!0,composed:!0}))}handleBlur(e){let t=e.target;this.dispatchEvent(new CustomEvent(`field-blur`,{detail:{key:this.field.key,value:t.value},bubbles:!0,composed:!0}))}get buttonFieldTemplate(){return a`
      <div class="left">
        <div class="label">${this.field.name}</div>
      </div>
      <div class="center button-control">
        <ia-button
          mode="danger"
          @click=${e=>this.handleChange(e,this.field)}
        >
          ${this.field.label}
        </ia-button>
      </div>
      <div class="right"></div>
    `}get inputFieldTemplate(){return a`<div class="single-col-row">
      <div class="label">${this.field.name}</div>
      <div class="description">${this.field.label}</div>
      <input
        type="${te(this.field.type===`select`?void 0:this.field.type)}"
        id="${this.field.key}"
        name="${this.field.key}"
        .value="${String(this.field.value??``)}"
        @input="${e=>this.handleChange(e,this.field)}"
        @blur="${this.handleBlur}"
      />
      ${this.field.type===`password`?this.passwordToggleTemplate:n}
    </div>`}get checkboxFieldTemplate(){return a`<input
      type="checkbox"
      id="${this.field.key}"
      name="${this.field.key}"
      .checked="${this.field.value===`on`}"
      @change="${e=>this.handleChange(e,this.field)}"
      @blur="${this.handleBlur}"
    />`}get dropdownFieldTemplate(){return a`<select
      id="${this.field.key}"
      name="${this.field.key}"
      .value="${String(this.field.value??``)}"
      @change="${e=>this.handleChange(e,this.field)}"
      @blur="${this.handleBlur}"
    >
      ${this.field.options?.map(e=>a`<option
            value="${e}"
            ?selected=${this.field?.value===e}
          >
            ${e}
          </option>`)}
    </select>`}get checkboxOrDropdownFieldTemplate(){return a` <div class="left">
        <div class="label">${this.field.name} ${this.infoIcon}</div>
      </div>
      <div class="center">
        ${this.field.type===`checkbox`?this.checkboxFieldTemplate:this.dropdownFieldTemplate}
      </div>
      <div class="right">
        <div>
          <label for="${te(this.field.key)}">
            ${this.field.label}
          </label>
        </div>
      </div>`}get infoIcon(){return a`<span
      class="desc-icon"
      @click="${()=>{this.showMobileDesc=!this.showMobileDesc}}"
    >
      &#9432;
    </span>`}get passwordToggleTemplate(){return a`<ia-password-toggle
      .passwordInput=${this.passwordField}
    ></ia-password-toggle>`}get fieldTemplate(){switch(this.field.type){case`button`:return this.buttonFieldTemplate;case`text`:case`email`:case`password`:return this.inputFieldTemplate;case`select`:case`checkbox`:return this.checkboxOrDropdownFieldTemplate;default:return this.inputFieldTemplate}}render(){return a`
      <div class="row">${this.fieldTemplate}</div>
      <div
        class="desc-mobile"
        ?show="${this.showMobileDesc&&window.innerWidth<768}"
      >
        ${this.field.label}
      </div>
    `}static get styles(){return e`
      :host {
        margin-bottom: 1rem;
      }
      .row {
        display: flex;
        align-items: var(--align-items--, start);
        justify-content: space-between;
        width: 100%;
      }
      .left {
        padding-right: 16px;
        width: 25%;
        display: flex;
        align-items: center;
      }
      .center {
        width: 5%;
        display: flex;
        justify-content: end;
      }
      .right {
        min-width: 120px;
        display: flex;
        align-items: center;
        justify-content: flex-start;
        width: 65%;
      }
      .right label {
        display: inline-block;
      }
      .label {
        font-weight: 500;
        margin-bottom: 0;
      }
      .desc-icon {
        margin-left: 0.5rem;
        cursor: pointer;
        font-size: 1.1em;
        vertical-align: middle;
        display: none;
        user-select: none;
      }
      .desc {
        font-size: 0.95em;
        color: #2c2c2c;
      }
      .desc-mobile {
        width: 100%;
        color: #2c2c2c;
        display: none;
      }
      @media (max-width: 768px) {
        .row {
          flex-direction: row;
          align-items: center;
        }
        .left {
          width: 60%;
          padding-right: 8px;
        }
        .left .label {
          font-weight: 500;
        }
        .right {
          width: 40%;
          min-width: unset;
          justify-content: flex-end;
        }
        .right {
          display: none;
        }
        .desc-icon {
          display: inline-block;
        }
        .desc-mobile[show] {
          display: block;
          margin-top: 0.5rem;
        }
      }

      .single-col-row {
        display: inline-block;
        width: 100%;
      }
      .single-col-row .label {
        font-weight: bold;
        margin-bottom: 4px;
        display: block;
      }
      .single-col-row input,
      .single-col-row select {
        display: inline-block;
        width: 75%;
        padding: 6px 12px;
        font-size: 14px;
        line-height: 1.42857;
        background-color: rgb(253, 253, 253);
        border: 1px solid rgb(204, 204, 204);
        border-radius: 4px;
        box-shadow: rgba(0, 0, 0, 0.075) 0px 1px 1px inset;
        transition:
          border-color 0.15s ease-in-out,
          box-shadow 0.15s ease-in-out;
      }
      .single-col-row .description {
        margin-bottom: 5px;
        white-space: pre-line;
      }

      select {
        max-width: 80px; /* prevent overflow on small screens */
      }

      /* Optional: make the dropdown wider when open (if supported by browser) */
      select:focus {
        width: auto;
      }

      /* You can also add text overflow for the selected item */
      select option {
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
      }

      .button-control {
        display: flex;
        align-items: center;
      }

      .button-control ia-button {
        --ia-theme-button-padding: 0 1.2rem;
        --ia-theme-button-height: 2.8rem;
        --ia-theme-font-size-standard: 1.2rem;
      }
    `}};f([t({type:String,reflect:!0})],K.prototype,`category`,void 0),f([t({type:String,reflect:!0})],K.prototype,`fieldName`,void 0),f([t({type:Object})],K.prototype,`field`,void 0),f([t({type:Boolean})],K.prototype,`hasDataUpdated`,void 0),f([s()],K.prototype,`showMobileDesc`,void 0),f([u(`#password`)],K.prototype,`passwordField`,void 0),K=f([l(`field-row`)],K);var q=class extends c{constructor(...e){super(...e),this.linkedProviders=[],this.pendingProvider=``,this.linkedProvidersFields=[],this.isProcessing=!1,this.providerUnlinkStatus={provider:``,status:``}}updated(e){e.has(`isProcessing`)&&this.isProcessing&&this.showUnlinkConfirmation()}willUpdate(e){e.has(`linkedProviders`)&&(this.linkedProvidersFields=this.linkedProviders.map(e=>({name:b(e),key:e,label:o(`Unlink account`),type:`button`})))}get providerUnlinkStatusTemplate(){let{provider:e,status:t}=this.providerUnlinkStatus;return!e||!t?null:a` <div class="${t}" role="alert">${t===`success`?`✔`:`✖`} ${t===`success`?S().GOOGLE_PROVIDER_UNLINKED:C().GOOGLE_PROVIDER_UNLINK_FAILED}</div> `}render(){let e=!!this.providerUnlinkStatus.status;return!this.linkedProvidersFields.length&&!e?a`<label>${o(`You have no linked accounts`)}</label>`:a`
      ${this.linkedProvidersFields.length?a`
            <div class="settings-grid">
              ${this.linkedProvidersFields.map(e=>a`
                  <div class="provider-row">
                    <field-row
                      category="unlink"
                      .field=${e}
                      @setting-changed=${e=>{e.stopPropagation(),this.pendingProvider=e.detail.key,this.showUnlinkConfirmation()}}
                    ></field-row>
                  </div>
                `)}
            </div>
          `:n}
      ${this.providerUnlinkStatusTemplate}
    `}showUnlinkConfirmation(){if(!this.modalManager)return;this.modalManager.setAttribute(`class`,`confirm-unlink-account`);let e=new p({title:a`${o(`Unlink account`)}`,headerColor:`#d9534f`,showCloseButton:!0,showHeaderLogo:!0,closeOnBackdropClick:!0});this.modalManager.showModal({config:e,customModalContent:this.unlinkProviderTemplate,userClosedModalCallback:()=>{}})}get unlinkProviderTemplate(){return a`
      <div class="confirm-content">
        <p style="font-size: 1.4rem">
          ${o(r`Are you sure you want to unlink your ${b(this.pendingProvider)} account?`)}
        </p>
        <div style="display: inline-flex; gap: 1rem">
          <ia-button
            mode="secondary"
            @click=${()=>this.modalManager?.closeModal()}
          >
            ${o(`Cancel`)}
          </ia-button>
          <ia-button
            mode="danger"
            .loadingText=${o(`Unlinking...`)}
            ?loading=${this.isProcessing}
            @click=${()=>this.unlinkProvider(this.pendingProvider)}
          >
            ${o(`Unlink account`)}
          </ia-button>
        </div>
      </div>
    `}async unlinkProvider(e){if(this.accountSettingsService){this.isProcessing=!0;try{let t=await this.accountSettingsService.unlinkProvider(e);this.providerUnlinkStatus={provider:e,status:t.success?`success`:`error`},t.success&&(this.linkedProviders=this.linkedProviders.filter(t=>t!==e))}catch{this.providerUnlinkStatus={provider:e,status:`error`}}finally{this.isProcessing=!1,this.modalManager?.closeModal()}}}static get styles(){return e`
      field-row {
        --align-items: center;
      }
      .settings-grid {
        display: flex;
        flex-direction: column;
        margin: 0 0 1rem 0;
      }
      .error {
        color: var(--error-color, #bb0505);
      }
      .success {
        color: var(--success-color, #219518);
      }
    `}};f([t({type:Object})],q.prototype,`accountSettingsService`,void 0),f([t({type:Array})],q.prototype,`linkedProviders`,void 0),f([t({type:Object})],q.prototype,`modalManager`,void 0),f([s()],q.prototype,`pendingProvider`,void 0),f([s()],q.prototype,`linkedProvidersFields`,void 0),f([s()],q.prototype,`isProcessing`,void 0),f([s()],q.prototype,`providerUnlinkStatus`,void 0),q=f([l(`linked-accounts`),i()],q);var J=class extends c{constructor(...e){super(...e),this.csrfToken=``,this.confirmDelete=!1,this.attemptToDelete=!1,this.showLoadingIndicator=!1}updated(e){if(e.has(`attemptToDelete`)&&this.attemptToDelete){let e=document.documentElement;e.scrollTo({top:e.scrollHeight,behavior:`smooth`})}}async deleteAccount(){this.showLoadingIndicator=!0,this.dispatchEvent(new CustomEvent(`deleteAccount`,{detail:{confirmDelete:this.confirmDelete}}))}render(){return a`
      <div class="account-deletion-container">
        <slot name="delete-section-header"></slot>
        ${this.attemptToDelete?this.deleteAccountTemplate:this.initialStateTemplate}
      </div>
    `}get initialStateTemplate(){return a`
      <div class="form-element delete-account-trigger">
        <p class="delete-description">
          If you no longer wish to use the Internet Archive / Open Library, you
          can
          <ia-button
            id="delete-link"
            mode="danger-link"
            @click=${()=>{this.attemptToDelete=!0}}
          >
            ${o(`Delete your account`)}</ia-button
          >.
        </p>
      </div>
    `}get deleteAccountTemplate(){return a`
      <div class="form-element delete-section">
        <div class="delete-header">
          <h3>Delete Internet Archive / Open Library account</h3>
          ${this.closeButtonTemplate}
        </div>

        <div class="content-wrapper">
          <div class="warning-message">
            <p>
              <strong>Please note:</strong> Items you've uploaded will remain on
              the Internet Archive.<br />
              If you wish to remove items, please do so before deleting your
              account.
            </p>

            <p>
              <a
                href="https://help.archive.org/help/how-do-i-delete-my-account/"
                target="_blank"
                rel="noopener"
                class="help-link"
              >
                Learn more about deleting your account and content
              </a>
            </p>
          </div>

          <div class="confirmation-section">
            <div class="checkbox-wrapper">
              <input
                type="checkbox"
                id="confirm-delete"
                name="confirm-delete"
                @change=${()=>{this.confirmDelete=!this.confirmDelete}}
              />
              <label for="confirm-delete" class="confirm-label">
                I understand this action cannot be reversed and want to
                permanently delete my account
              </label>
            </div>

            <ia-button
              id="delete-button"
              mode="danger"
              ?loading=${this.showLoadingIndicator}
              .loadingText=${o(`Deleting Account...`)}
              ?disabled=${!this.confirmDelete}
              @click=${this.deleteAccount}
            >
              ${o(`Permanently delete my account`)}
            </ia-button>
          </div>
        </div>
      </div>
    `}get closeButtonTemplate(){let e=o(`Close delete section`);return a`
      <button
        class="cancel-button"
        title=${e}
        aria-label=${e}
        @click=${()=>{this.attemptToDelete=!1,this.confirmDelete=!1}}
      >
        <img src=${ce} aria-hidden="true" />
      </button>
    `}static get styles(){return e`
      :host {
        display: block;
        margin-top: 2rem;
        margin-bottom: 1rem;
      }

      h3 {
        color: #333;
        font-size: 1.4rem;
        margin-top: 0;
        margin-bottom: 0.5rem;
        font-weight: 600;
      }

      p {
        margin: 0.5rem 0;
        line-height: 1.5;
        color: #333;
        font-size: 1.4rem;
      }

      .delete-description {
        margin-bottom: 1rem;
      }

      ia-button#delete-link {
        padding: 0.5rem 0;
        display: inline-block;
        --ia-theme-button-height: 3rem;
      }

      /* Delete confirmation section */
      .delete-section {
        border: 1px solid #d9534f;
        border-radius: 4px;
        padding: 1.5rem;
        box-shadow: 0 2px 5px rgba(0, 0, 0, 0.05);
        margin-top: 1rem;
      }

      .delete-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 1rem;
        padding-bottom: 1.5rem;
        border-bottom: 1px solid #f1c9c7;
        position: relative;
      }

      .delete-header h3 {
        color: #d9534f;
        margin: 0;
      }

      .cancel-button {
        position: absolute;
        top: 0;
        right: 0;
        width: 20px;
        height: 20px;
        padding: 0;
        border: none;
        background: none;
        appearance: none;
        cursor: pointer;
      }

      .content-wrapper {
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
      }

      .warning-message {
        background-color: #fef7f7;
        border-left: 3px solid #d9534f;
        padding: 1rem;
        margin-bottom: 0.5rem;
      }

      .help-link {
        color: #337ab7;
        text-decoration: none;
        font-weight: 500;
      }

      .help-link:hover {
        text-decoration: underline;
      }

      .confirmation-section {
        display: inline;
      }

      .checkbox-wrapper {
        display: flex;
        align-items: flex-start;
        gap: 0.5rem;
        margin-bottom: 0.5rem;
      }

      input[type='checkbox'] {
        margin-top: 0.3rem;
      }

      .confirm-label {
        font-weight: 500;
        color: #333;
        font-size: 1.4rem;
        line-height: 1.5;
        margin-bottom: 1rem;
      }
    `}};f([t({type:String})],J.prototype,`csrfToken`,void 0),f([s()],J.prototype,`confirmDelete`,void 0),f([s()],J.prototype,`attemptToDelete`,void 0),f([s()],J.prototype,`showLoadingIndicator`,void 0),J=f([l(`delete-account`),i()],J);var Y=class extends c{render(){return a`<div class="data-updates">${this.updatesTemplate}</div>`}get updatesTemplate(){return this.fields?Object.values(this.fields??{}).map(e=>e.msg?a`<span class="success-field">✔ ${e.msg}</span>`:e.error?a`<span class="error-field">
            ✖ ${ee(ie.sanitize(e.error,{ALLOWED_TAGS:[`a`]}))}
          </span>`:a``):a``}static get styles(){return e`
      :host {
        --success-color--: var(--success-color, #219518);
        --error-color--: var(--error-color, #bb0505);
      }

      .data-updates {
        display: grid;
        background: rgb(238, 253, 238);
        width: fit-content;
      }

      /* Individual message styles */
      span {
        display: block;
        padding: 1rem;
        border-left: 5px solid green;
      }
      span.error-field {
        color: var(--error-color--);
        border-left: 5px solid var(--error-color--);
      }
      span.error-field a {
        color: inherit;
      }

      span.success-field {
        color: var(--success-color--);
        border-left: 5px solid var(--success-color--);
      }
    `}};f([t({type:Object})],Y.prototype,`fields`,void 0),Y=f([l(`updates-summary`)],Y);var X=class extends c{constructor(...e){super(...e),this.selectedMailingLists=[],this.fields=[]}willUpdate(e){e.has(`mailingLists`)&&this.mailingLists&&(this.fields=Object.entries(this.mailingLists).filter(([,e])=>e.public).map(([,e])=>({name:e.name,key:e.key,label:e.short_desc,type:`checkbox`,value:this.selectedMailingLists.includes(e.key)?`on`:`off`,public:e.public,category:`mailing`})))}render(){return a`
      <div class="settings-grid">
        <p>
          ${o(`Stay up to date with what's happening at the Internet Archive by signing up for our free newsletters.`)}
        </p>
        ${this.fields.map(e=>a`
            <field-row .category=${`mailing`} .field=${e}> </field-row>
          `)}
      </div>
    `}static get styles(){return[e`
        .settings-grid {
          display: flex;
          flex-direction: column;
          margin: 0 0 2rem 0;
        }
        .settings-grid field-row:last-child {
          margin: 0;
        }
        .settings-grid p {
          margin: 0 0 1rem 0;
        }
        @media (max-width: 768px) {
          .settings-grid p {
            display: none;
          }
        }
      `]}};f([t({type:Object})],X.prototype,`mailingLists`,void 0),f([t({type:Array})],X.prototype,`selectedMailingLists`,void 0),f([s()],X.prototype,`fields`,void 0),X=f([l(`mailing-lists-settings`),i()],X);var Z=class extends c{constructor(...e){super(...e),this.loanHistoryFlag=`private`,this.anyStaff=!1,this.displaySettings=[],this.searchSettings=[],this.defaultFields=[{name:o(`Public Borrowing History`),key:`public_borrowing_history`,label:o(`Allow the public to see your borrowing history`),type:`checkbox`,value:`off`,default_value:`on`,is_defaulted:!1,category:`display`}]}willUpdate(e){(e.has(`preferences`)||e.has(`loanHistoryFlag`))&&(this.mapPreferences(),this.mergeDefaultFields(),this.categorizeFields())}mapPreferences(){this.fields=this.preferences?.map(e=>{let t=e.policy_details?.display_properties?.pretty_name||e.preference,n=e.policy_details?.display_properties?.pretty_description||``,r=e.preference.toLowerCase().replace(/\s+/g,`_`),i=e.policy_details?.preference_properties?.allowed_values||[],a=e.policy_details?.preference_properties?.default_value||`off`,o=i.length===2&&i.includes(`on`)&&i.includes(`off`)?`checkbox`:`select`,s={name:t,key:r,label:n,type:o,value:e.value,default_value:a,is_defaulted:e.is_defaulted,category:e.category};return o===`select`&&(s.options=i),s})}mergeDefaultFields(){let e=this.loanHistoryFlag===`public`?`on`:`off`;if(this.fields?.some(e=>e.key===`public_borrowing_history`))this.fields=this.fields?.map(t=>t.key===`public_borrowing_history`?{...t,value:e}:t);else{let t=this.defaultFields.map(t=>({...t,value:e}));this.fields=[...t,...this.fields??[]]}}categorizeFields(){this.displaySettings=this.fields?.filter(e=>e.category===`display`)||[],this.searchSettings=this.fields?.filter(e=>e.category===`search`)||[]}render(){return a`
      ${this.displaySettings.length>0?this.displaySettingsTemplate:n}
      ${this.searchSettings.length>0?this.searchSettingsTemplate:n}
    `}get displaySettingsTemplate(){return a`
      <div class="settings-grid">
        <slot name="display-settings-header"></slot>
        ${this.displaySettings?.map(e=>a`
            <field-row .category=${`display`} .field=${e}></field-row>
          `)}
      </div>
    `}get searchSettingsTemplate(){return a`
      <div class="settings-grid">
        <slot name="search-settings-header"></slot>
        ${this.searchSettings?.map(e=>a`
            <field-row .category=${`search`} .field=${e}></field-row>
          `)}
      </div>
    `}static get styles(){return[e`
        .settings-grid {
          display: flex;
          flex-direction: column;
          margin: 0 0 2rem 0;
        }
        .settings-grid field-row:last-child {
          margin: 0;
        }
      `]}};f([t({type:Array})],Z.prototype,`preferences`,void 0),f([t({type:String})],Z.prototype,`loanHistoryFlag`,void 0),f([t({type:Boolean})],Z.prototype,`anyStaff`,void 0),f([s()],Z.prototype,`displaySettings`,void 0),f([s()],Z.prototype,`searchSettings`,void 0),Z=f([l(`preference-settings`),i()],Z);var Q=class extends c{constructor(...e){super(...e),this.userData={email:``,screenname:``,identifier:``},this.errors={},this.hasDataUpdated=!1,this.fields=[]}willUpdate(e){e.has(`userData`)&&(this.fields=[{name:o(`Change Screenname`),key:`screenname`,label:o(`Changing your screenname will not change your user ID.
You cannot re-use a screenname once it has been changed.`),type:`text`,value:this.userData?.screenname,category:`account`},{name:o(`Change Email`),key:`email`,label:o(`You cannot re-use an email address once it has been changed.`),type:`email`,value:this.userData?.email,category:`account`},{name:o(`Change Internet Archive / Open Library Password`),key:`password`,label:``,type:`password`,value:``,category:`account`}])}render(){return a`
      <div class="settings-field-group">
        ${this.fields.map(e=>a`
            <field-row
              .category=${`account`}
              .field="${e}"
              ?hasDataUpdated=${this.hasDataUpdated}
            ></field-row>
            ${this.errors[e.key]?a`<span class="error-field">${this.errors[e.key]}</span>`:n}
          `)}
      </div>
    `}static get styles(){return e`
      :host {
        --error-color--: var(--error-color, #bb0505);
      }

      .settings-field-group {
        display: flex;
        flex-direction: column;
        gap: 5px;
      }
      .error-field {
        color: var(--error-color--);
        font-size: 1.2rem;
        margin-bottom: 9px;
        display: block;
        padding-left: 0.5rem;
      }
    `}};f([t({type:Object})],Q.prototype,`userData`,void 0),f([t({type:Object})],Q.prototype,`errors`,void 0),f([t({type:Boolean})],Q.prototype,`hasDataUpdated`,void 0),f([s()],Q.prototype,`fields`,void 0),Q=f([l(`text-fields-group`),i()],Q);var $=class extends c{constructor(...e){super(...e),this.userData={email:``,screenname:``,identifier:``},this.linkedProviders=[],this.settingsViewName=`loading`,this.mgcToken=``,this.fieldsError={},this.updatedFields={},this.apiHasExecuted=!1,this.saveButtonDisabled=!0,this.showLoadingIndicator=!0,this.isStickyHeader=!1,this.mgcViewName=`welcome`,this.mgcController=null,this.googleClientId=``,this.settingsServicePath=``,this.archiveApiUrl=``,this.eventListeners={scroll:`scroll`}}connectedCallback(){super.connectedCallback(),this.bindEvents()}disconnectedCallback(){super.disconnectedCallback(),this.unbindEvents()}bindEvents(){this.scrollHandler=le(()=>{this.isStickyHeader=window.scrollY>this.offsetTop},20),window.addEventListener(this.eventListeners.scroll,this.scrollHandler)}unbindEvents(){window.removeEventListener(this.eventListeners.scroll,this.scrollHandler)}firstUpdated(){this.setupAppServicesDependents()}willUpdate(e){e.has(`mgcToken`)&&this.mgcToken&&this.mgcController&&(this.mgcController.mgcToken=this.mgcToken)}async loadMonthlyGivingCircleIntoSlot(){let e=await this.appServices?.get(),t=await e?.lightDOMTunnel.get();if(await t?.setSlotContent(`center-of-form`,a`<ia-monthly-giving-circle-controller></ia-monthly-giving-circle-controller>`),t&&(await t.getSlot(`center-of-form`).then(e=>{if(!e)return;if(!e.querySelector(`ia-monthly-giving-circle-controller`)){console.warn(`No mgc controller found in slot`);return}let t=e.querySelector(`ia-monthly-giving-circle-controller`);this.mgcController=t}),this.mgcController)){this.mgcController.appServices=this.appServices,this.mgcController.mgcToken=this.mgcToken,this.mgcController.patronEmail=this.userData.email;let t=await e?.abtestManager.get();this.mgcController.canEditPaymentMethod=await t?.variantFor(`MGCPaymentMethod`)===`On`,this.mgcViewName=this.mgcViewName,this.mgcController.addEventListener(`viewChanged`,e=>this.handleViewChanged(e))}}async setupAppServicesDependents(){this.sendAnalyticsEvent(T.pageView,`/account/settings`);try{let e=await this.appServices?.get();this.pageMetadataService=await e?.pageMetadataService.get(),this.analyticsHandler=await e?.analyticsHandler.get(),this.modalManager=await e?.modalManager.get(),await e?.appConfig.get().then(e=>{this.googleClientId=e?.googleClientId,this.archiveApiUrl=e?.archiveApiUrl,this.settingsServicePath=e?.accountSettingsServicePath});let t=await e?.fetchHandler.get();if(!t)return;this.accountSettingsService=new fe(this.settingsServicePath,t),await this.fetchAccountSettings(),await this.setupPageMetadata()}catch(e){m(`Error setting up account settings service:`,e)}}async setupPageMetadata(){this.pageMetadataService?.setTitle(o(`Change Account Settings`)),this.pageMetadataService?.addLink({rel:`canonical`,path:`/account/settings`}),this.pageMetadataService?.setMetaTag({property:`og:type`,content:`website`}),this.pageMetadataService?.setMetaTag({property:`og:url`,content:`https://archive.org/account/settings`})}async fetchAccountSettings(){try{this.settingsViewName=`loading`;let e=(await this.accountSettingsService.fetchAccountSettings()).value;this.fetchedSettings=e,this.linkedProviders=e?.linkedProviders??[],this.userData=e?.userData}catch(e){m(`Error fetching account settings:`,e)}finally{this.settingsViewName=`verification`}}updated(e){this.handleLoadingAndResponseReset(e),this.handleFieldErrors(e),this.handleSaveButtonState(e),e.has(`settingsViewName`)&&this.settingsViewName===`settings`&&!this.mgcController&&this.loadMonthlyGivingCircleIntoSlot(),e.has(`settingsViewName`)&&(this.settingsViewName===`fullscreenTakeover`&&window.scrollTo(0,0),e.get(`settingsViewName`)===`fullscreenTakeover`&&this.mgcController?.scrollIntoView({behavior:`smooth`,block:`start`,inline:`start`}))}handleLoadingAndResponseReset(e){(e.has(`userData`)||e.has(`selectedMailingLists`))&&(this.showLoadingIndicator=!1,this.updatedFields={})}handleFieldErrors(e){e.has(`fieldsError`)&&this.accountSettingsService?.hasFieldError()&&(this.saveButtonDisabled=!0)}handleSaveButtonState(e){e.has(`userData`)&&!this.accountSettingsService?.hasFieldError()&&(this.saveButtonDisabled=!this.saveButtonDisabled)}handleSettingsChanges(e){this.accountSettingsService.handleSettingsChanges(e),this.changeSaveButtonState()}handleFieldBlur(e){let t=e.detail.key;t===`screenname`?this.validateScreenname():t===`email`?this.validateEmail():t===`password`&&this.validatePassword()}sendAnalyticsEvent(e,t){this.analyticsHandler?.sendEvent({category:w,action:e,label:t})}render(){return a`
      <main id="maincontent" class="${this.settingsViewName===`fullscreenTakeover`?`show-fullscreen`:``}">
        <div class="container">
          ${this.settingsViewName===`loading`||this.settingsViewName===`verification`?a`
                ${this.settingsViewName===`loading`?this.loadingTemplate:n}
                ${this.settingsViewName===`verification`?this.verificationTemplate:n}
              `:a`${this.settingsTemplate}`}
        </div>
      </main>
    `}get loadingTemplate(){return a`
      <div class="initial-load-template">
        <ia-status-indicator
          mode="loading"
          class="loader-icon"
        ></ia-status-indicator>
        <p>${o(`Loading account settings...`)}</p>
      </div>
    `}async handleDeleteAccount(e){let t=await this.accountSettingsService.deleteAccount(e.detail.confirmDelete);this.sendAnalyticsEvent(T.deleteAccount,t.success?`succeeded`:`failed`),t.success?window.location.href=`/logout`:this.updatedFields={api:{error:C().DELETE_FAILED}}}changeSaveButtonState(){this.saveButtonDisabled=!!this.accountSettingsService.hasFieldError()}async validateScreenname(){let e=await this.accountSettingsService.validateScreenname();this.fieldsError={...this.fieldsError,screenname:e}}async validateEmail(){let e=await this.accountSettingsService.validateEmail();this.fieldsError={...this.fieldsError,email:e}}async validatePassword(){let e=this.accountSettingsService.validatePassword();this.fieldsError={...this.fieldsError,password:e}}async saveAccountSettings(e){if(this.showLoadingIndicator=!0,this.saveButtonDisabled=!0,x(e),this.updatedFields={},await this.iaPicUploader?.onSubmit(),!this.accountSettingsService.hasFieldError()){try{let e=await this.accountSettingsService.handleSaveAccountSettings();e.success&&e.value&&Object.keys(e.value).length?this.updatedFields={...this.updatedFields,...e.value}:e.success||(this.updatedFields={...this.updatedFields,api:{error:e.error||C().API_ERROR}}),this.apiHasExecuted=!0}catch{this.updatedFields={...this.updatedFields,api:{error:`An error occurred while saving settings`}}}finally{this.userData.password=``,Object.keys(this.updatedFields).length&&this.sendAnalyticsEventsOnSave()}setTimeout(()=>{this.showLoadingIndicator=!1},100)}}profilePictureUploaded(e){this.saveButtonDisabled=!0;let t=!e?.detail?.hasError;this.setFieldUpdateState(`file`,t,S().PROFILE_PICTURE,C().UPLOAD_FAILED_GENERIC)}handleFileChange(e){this.fieldsError={...this.fieldsError,picture:e};let t=!!e;this.saveButtonDisabled=t}setFieldUpdateState(e,t,n,r){this.updatedFields={...this.updatedFields,[e]:t?{msg:n}:{error:r}}}sendAnalyticsEventsOnSave(){if(!this.updatedFields)return;let e={email:`email`,screenname:`screenname`,password:`password`,preference:`preferences`,mailing:`newsletter`,file:`picture`,unlink:`third-party-unlink`,api:`api-error`},t=[],n=[];Object.entries(this.updatedFields).forEach(([r,i])=>{if(!i)return;let a=e[r]??r;i.msg&&t.push(a),i.error&&n.push(a)}),t.length&&this.sendAnalyticsEvent(T.saved,t.join(`|`)),n.length&&this.sendAnalyticsEvent(T.saveFailed,n.join(`|`))}get verificationTemplate(){return a` <ia-verification
      .appServices=${this.appServices}
      .analyticsHandler=${this.analyticsHandler}
      .settingsServicePath=${this.settingsServicePath}
      .identifier=${this.userData.identifier??``}
      .email=${this.userData.email}
      .csrfToken=${this.fetchedSettings?.csrfToken??``}
      .archiveApiUrl=${this.archiveApiUrl}
      .googleClientId=${this.googleClientId}
      @iaAuthenticated=${e=>{this.settingsViewName=`settings`,this.mgcToken=e.detail.token}}
      @IAThirdPartyAuthVerified=${()=>{this.sendAnalyticsEvent(T.authenticated,`third-party`),m(`IAThirdPartyAuth:verifiedLogin`),this.settingsViewName=`settings`}}
      @mgcTokenReceived=${e=>{m(`IAThirdPartyAuth:MGCToken`),this.mgcToken=e.detail.token}}
    >
    </ia-verification>`}get settingsTemplate(){return a`<div
      class="settings-template ${this.isStickyHeader?`sticky-header`:``} "
    >
      <form id="form" name="account-settings" method="post" autocomplete="off">
        <div
          class="form-element header ${this.showLoadingIndicator?`pointer-none`:``}"
        >
          <div class="actions">
            <h2>${o(`Account Settings`)}</h2>
            <ia-button
              mode="secondary"
              @click=${()=>{setTimeout(()=>{window.location.href=`/`},10)}}
            >
              ${o(`Cancel`)}
            </ia-button>
            <ia-button
              class="save-button"
              mode="primary"
              @click="${this.saveAccountSettings}"
              ?disabled=${this.saveButtonDisabled}
              ?loading=${this.showLoadingIndicator}
            >
              ${o(`Save Changes`)}
            </ia-button>
          </div>
          <updates-summary .fields=${this.updatedFields}></updates-summary>
        </div>

        <div class="body-content">
          <!-- Profile picture section -->
          <div class="form-element pic-uploader-section">
            <label>${o(`Change Profile Picture`)}</label>
            <ia-pic-uploader
              .appServices=${this.appServices}
              .identifier=${this.userData.identifier??``}
              .picture=${this.fetchedSettings?.profilePicture??``}
              .variant=${`compact`}
              ?lookingAtMyAccount=${!0}
              @fileChanged=${e=>{this.handleFileChange(e.detail.error)}}
              @fileUploaded=${e=>{this.profilePictureUploaded(e)}}
            ></ia-pic-uploader>
            <span class="error-field">${this.fieldsError.picture}</span>
          </div>

          <!-- text fields section -->
          <div class="form-element">
            <text-fields-group
              .userData="${this.userData}"
              .errors="${this.fieldsError}"
              ?hasDataUpdated=${this.apiHasExecuted}
              @setting-changed="${this.handleSettingsChanges}"
              @field-blur="${this.handleFieldBlur}"
            ></text-fields-group>
          </div>

          <!-- Monthly donation component element -->
          <div
            class="form-element needs-fullscreen"
            id="monthly-giving-circle-slot"
          >
            <slot name="center-of-form"></slot>
          </div>

          <!-- Preferences settings component -->
          <div class="form-element preference-settings">
            <preference-settings
              .preferences=${this.fetchedSettings?.preferenceSettings}
              .loanHistoryFlag=${this.fetchedSettings?.loanHistoryFlag??`private`}
              ?anyStaff=${this.fetchedSettings?.anyStaff??!1}
              @setting-changed=${this.handleSettingsChanges}
            >
              <h3 slot="display-settings-header">${o(`Display Settings`)}</h3>
              <h3 slot="search-settings-header">${o(`Search Settings`)}</h3>
            </preference-settings>
          </div>

          <!-- Mailing lists settings component -->
          <div class="form-element mailing-lists-settings">
            <h3>${o(`Newsletter Subscriptions`)}</h3>
            <mailing-lists-settings
              .mailingLists=${this.fetchedSettings?.mailingLists}
              .selectedMailingLists=${this.fetchedSettings?.selectedMailingLists??[]}
              @setting-changed=${this.handleSettingsChanges}
            ></mailing-lists-settings>
          </div>

          <!-- Linked accounts section -->
          <div class="form-element linked-accounts-settings">
            <h3>${o(`Linked 3rd Party Accounts`)}</h3>
            <linked-accounts
              .accountSettingsService=${this.accountSettingsService}
              .linkedProviders=${this.linkedProviders}
              .modalManager=${this.modalManager}
            ></linked-accounts>
          </div>

          <!-- Delete account section -->
          <div class="form-element">
            <delete-account
              .csrfToken=${this.fetchedSettings?.csrfToken??``}
              @deleteAccount=${this.handleDeleteAccount}
              ><h3 slot="delete-section-header">
                ${o(`Delete Internet Archive / Open Library Account`)}
              </h3>
            </delete-account>
          </div>

          ${this.userData?.isAdmin?a`<admin-links></admin-links>`:n}
        </div>
      </form>
    </div>`}async handleViewChanged(e){switch(this.mgcViewName=e.detail,m(`MGC view changed to:`,this.mgcViewName),this.mgcViewName){case`receipts`:case`editPlan`:this.settingsViewName=`fullscreenTakeover`;break;case`plans`:case`welcome`:this.settingsViewName=`settings`;break}}static get styles(){return[ue,e`
        :host {
          display: block;
          padding: 25px;
          font-size: 1.4rem;
          min-height: calc(100vh - 50px);
        }

        :host([settingsViewName='fullscreenTakeover']) {
          overflow: hidden;
          height: 100%;
          position: relative;
        }

        slot[name='center-of-form'] {
          display: flex;
          align-items: center;
        }

        ::slotted(#form-center) {
          margin-bottom: 15px;
          display: flex;
        }

        ia-verification {
          width: 90vw;
          max-width: 525px;
        }

        :host([settingsViewName='fullscreenTakeover']) .needs-fullscreen {
          position: absolute;
          background: white;
          padding: 30px 0 0 30px;
          z-index: 2;
          left: 0px;
          overflow: auto;
          width: 100%;
          display: grid;
          justify-content: center;
          top: 0;
          bottom: 0;
          margin: 0;
        }

        :host([settingsViewName='fullscreenTakeover'])
          .needs-fullscreen
          slot[name='center-of-form'] {
          display: block;
        }

        slot[name='center-of-form'] {
          width: 100%;
          max-width: 600px;
        }
      `]}};f([t({type:Object})],$.prototype,`appServices`,void 0),f([t({type:Object})],$.prototype,`userData`,void 0),f([t({type:Array})],$.prototype,`linkedProviders`,void 0),f([t({type:String,reflect:!0})],$.prototype,`settingsViewName`,void 0),f([s()],$.prototype,`mgcToken`,void 0),f([s()],$.prototype,`fetchedSettings`,void 0),f([s()],$.prototype,`fieldsError`,void 0),f([s()],$.prototype,`updatedFields`,void 0),f([s()],$.prototype,`apiHasExecuted`,void 0),f([s()],$.prototype,`saveButtonDisabled`,void 0),f([s()],$.prototype,`showLoadingIndicator`,void 0),f([s()],$.prototype,`isStickyHeader`,void 0),f([s()],$.prototype,`mgcViewName`,void 0),f([s()],$.prototype,`mgcController`,void 0),f([u(`ia-pic-uploader`)],$.prototype,`iaPicUploader`,void 0),$=f([l(`account-settings`),i()],$);var we=$;export{we as default};