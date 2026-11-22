import{n as e}from"./rolldown-runtime-QTnfLwEv.js";import{A as t,C as n,E as r,O as i,S as a,T as o,d as s,f as c,k as l,m as u,w as d,x as f,y as p}from"./vendor-lit-MQ4DzmMc.js";import{s as m}from"./vendor-shared-62wh2Xkx.js";var h=l`
<svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg"><path d="m51.5960452 0c5.420012 0 6.7920618 2.72313378 8.1300179 9.28391016 2.9793915 19.21608984-2.9793915 28.94849474 0 31.58229234 1.2215505 1.079857 1.8678662.7924226 3.2997314.0773518l.2153069-.1075882c2.1016905-1.0490161 5.8499713-2.8359661 14.9013153-2.8359661l2.3393989.00075c14.0694555.01425 14.2569231.29925 18.0062757 5.99925 0 3.2648986.9924719 9.5-3.9698878 9.5 4.9623597 0 7.9397755 15.5 0 15.5 7.9397755 0 6.9473035 15.5-.9924719 15.5 7.9397754 0 5.9548316 12 2.9774158 13-1.1342536.5714286-1.9876793 1.0011812-2.627102 1.3222794l-.3279542.1645673c-1.5889262.796641-1.5747062.7780161-1.4530457.6832252l.0244872-.0190992c.0041709-.0032756.0083532-.0065808.0124967-.0098908l.0239934-.0197196c.1060465-.0904428.1029348-.1457697-1.111471.3786377h-55.0821921c-3.3082398-3.0266004-4.9623597-5.0266004-4.9623597-6v-33.4737525c5.5882429-8.3508317 10.6469206-21.2754909 15.1760333-38.7739777v-17.92948326c0-2.54852436 1.8066707-3.82278654 5.4200119-3.82278654zm-27.5960452 56c2.209139 0 4 1.790861 4 4v36c0 2.209139-1.790861 4-4 4h-20c-2.209139 0-4-1.790861-4-4v-36c0-2.209139 1.790861-4 4-4z" fill-rule="evenodd"/></svg>`,g=l`
<svg viewBox="0 0 100 100"
  xmlns="http://www.w3.org/2000/svg">
  <path d="m51.5960452 0c5.420012 0 6.7920618 2.72313378 8.1300179 9.28391016 2.9793915 19.21608984-2.9793915 28.94849474 0 31.58229234 1.2215505 1.079857 1.8678662.7924226 3.2997314.0773518l.2153069-.1075882c2.1016905-1.0490161 5.8499713-2.8359661 14.9013153-2.8359661l2.3393989.00075c14.0694555.01425 14.2569231.29925 18.0062757 5.99925 0 3.2648986.9924719 9.5-3.9698878 9.5 4.9623597 0 7.9397755 15.5 0 15.5 7.9397755 0 6.9473035 15.5-.9924719 15.5 7.9397754 0 5.9548316 12 2.9774158 13-1.1342536.5714286-1.9876793 1.0011812-2.627102 1.3222794l-.3279542.1645673c-1.5889262.796641-1.5747062.7780161-1.4530457.6832252l.0244872-.0190992c.0041709-.0032756.0083532-.0065808.0124967-.0098908l.0239934-.0197196c.1060465-.0904428.1029348-.1457697-1.111471.3786377h-55.0821921c-3.3082398-3.0266004-4.9623597-5.0266004-4.9623597-6v-33.4737525c5.5882429-8.3508317 10.6469206-21.2754909 15.1760333-38.7739777v-17.92948326c0-2.54852436 1.8066707-3.82278654 5.4200119-3.82278654zm-27.5960452 56c2.209139 0 4 1.790861 4 4v36c0 2.209139-1.790861 4-4 4h-20c-2.209139 0-4-1.790861-4-4v-36c0-2.209139 1.790861-4 4-4z" fill-rule="evenodd" transform="matrix(-1 0 0 -1 100 100)"/>
</svg>
`,_=class extends o{constructor(){super(...arguments),this.prompt=`Do you find this feature useful?`,this.buttonText=`Beta`,this.displayMode=`button`,this.isOpen=!1,this.processing=!1,this.popupTopX=0,this.popupTopY=0,this.voteSubmitted=!1,this.voteNeedsChoosing=!1,this.resizingElement=document.body}render(){return i`
      <div id="container">
        ${this.displayMode===`vote-prompt`?this.votePromptDisplay:this.singleButtonDisplay}
      </div>
    `}firstUpdated(){this.boundEscapeListener=this.handleEscape.bind(this),this.boundScrollListener=this.handleScroll.bind(this)}updated(e){if(e.has(`vote`)&&this.vote&&(this.error=void 0,this.voteNeedsChoosing=!1),e.has(`resizeObserver`)){let t=e.get(`resizeObserver`);this.disconnectResizeObserver(t)}}handleResize(){this.isOpen&&this.positionPopup()}handleScroll(){this.isOpen&&this.positionPopup()}disconnectedCallback(){this.removeEscapeListener(),this.disconnectResizeObserver(this.resizeObserver)}disconnectResizeObserver(e){(e??this.resizeObserver)?.removeObserver({handler:this,target:this.resizingElement})}setupResizeObserver(){this.resizeObserver&&this.resizeObserver.addObserver({handler:this,target:this.resizingElement})}async setupRecaptcha(){this.recaptchaManager&&(this.recaptchaWidget=await this.recaptchaManager.getRecaptchaWidget())}resetState(){this.vote=void 0,this.voteSubmitted=!1,this.error=void 0,this.voteNeedsChoosing=!1,this.comments.value=``}async showPopup(){this.voteSubmitted&&this.displayMode===`button`||(this.setupResizeObserver(),this.setupScrollObserver(),this.setupEscapeListener(),this.positionPopup(),this.isOpen=!0,await this.setupRecaptcha())}closePopup(){this.disconnectResizeObserver(),this.stopScrollObserver(),this.removeEscapeListener(),this.isOpen=!1}positionPopup(){let e=this.container.getBoundingClientRect(),t=this.popup.getBoundingClientRect(),n=window.innerWidth,r=window.innerHeight,i=n/2,a=r/2;e.left<i?this.popupTopX=e.right-20:this.popupTopX=e.left+20-t.width,this.popupTopX=Math.max(0,this.popupTopX),this.popupTopX+t.width>n&&(this.popupTopX=n-t.width),e.top<a?this.popupTopY=e.bottom-10:this.popupTopY=e.top+10-t.height}handleEscape(e){e.key===`Escape`&&this.cancel(e)}setupEscapeListener(){document.addEventListener(`keyup`,this.boundEscapeListener)}removeEscapeListener(){document.removeEventListener(`keyup`,this.boundEscapeListener)}setupScrollObserver(){document.addEventListener(`scroll`,this.boundScrollListener)}stopScrollObserver(){document.removeEventListener(`scroll`,this.boundScrollListener)}get singleButtonDisplay(){return i`
      <button
        id="beta-button"
        @click=${this.showPopup}
        tabindex="0"
        ?disabled=${this.disabled}
      >
        <span id="button-text">${this.buttonText}</span>
        <span
          class="beta-button-thumb upvote-button ${this.voteSubmitted?this.upvoteButtonClass:``}"
          >${h}</span
        >
        <span
          class="beta-button-thumb downvote-button ${this.voteSubmitted?this.downvoteButtonClass:``}"
          id="beta-button-thumb-down"
          >${g}</span
        >
      </button>
      ${this.popupTemplate}
    `}get votePromptDisplay(){return i`
      <form
        @submit=${this.submit}
        ?disabled=${this.processing||this.voteSubmitted}
      >
        <div class="prompt">
          <span class="prompt-text">${this.prompt}</span>
          <label
            tabindex="0"
            role="button"
            aria-pressed=${this.upvoteSelected}
            @keyup=${this.upvoteKeypressed}
            class="vote-button upvote-button ${this.upvoteButtonClass}"
          >
            <input
              type="radio"
              name="vote"
              value="up"
              @click=${this.upvoteButtonSelected}
              ?checked=${this.upvoteSelected}
            />
            ${h}
          </label>

          <label
            tabindex="0"
            role="button"
            aria-pressed=${this.downvoteSelected}
            @keyup=${this.downvoteKeypressed}
            class="vote-button downvote-button ${this.downvoteButtonClass}"
          >
            <input
              type="radio"
              name="vote"
              value="down"
              @click=${this.downvoteButtonSelected}
              ?checked=${this.downvoteSelected}
            />
            ${g}
          </label>
          <button id="comment-button" type="button" @click=${this.showPopup}>
            Leave a comment
          </button>
        </div>
      </form>
      ${this.popupTemplate}
    `}get popupTemplate(){return i`
      <div
        id="popup-background"
        class=${this.isOpen?`open`:`closed`}
        @click=${this.backgroundClicked}
        @keyup=${this.backgroundClicked}
      >
        <div
          id="popup"
          style="left: ${this.popupTopX}px; top: ${this.popupTopY}px"
        >
          <form
            @submit=${this.submit}
            id="form"
            ?disabled=${this.processing||this.voteSubmitted}
          >
            <div class="prompt">
              <div class="prompt-text">${this.prompt}</div>
              <label
                tabindex="0"
                role="button"
                aria-pressed=${this.upvoteSelected}
                @click=${this.upvoteButtonSelected}
                @keyup=${this.upvoteKeypressed}
                class="vote-button upvote-button ${this.upvoteButtonClass} ${this.chooseVoteErrorClass}"
              >
                <input
                  type="radio"
                  name="vote"
                  value="up"
                  @click=${this.upvoteButtonSelected}
                  ?checked=${this.upvoteSelected}
                />
                ${h}
              </label>

              <label
                tabindex="0"
                role="button"
                aria-pressed=${this.downvoteSelected}
                @click=${this.downvoteButtonSelected}
                @keyup=${this.downvoteKeypressed}
                class="vote-button downvote-button ${this.downvoteButtonClass} ${this.chooseVoteErrorClass}"
              >
                <input
                  type="radio"
                  name="vote"
                  value="down"
                  @click=${this.downvoteButtonSelected}
                  ?checked=${this.downvoteSelected}
                />
                ${g}
              </label>
            </div>
            <div>
              <textarea
                placeholder="Comments (optional)"
                id="comments"
                tabindex="0"
                ?disabled=${this.processing}
              ></textarea>
            </div>
            ${this.error?i`<div id="error">${this.error}</div>`:r}
            <div id="actions">
              <button
                @click=${this.cancel}
                id="cancel-button"
                class="cta-button"
                tabindex="0"
                ?disabled=${this.processing}
              >
                Cancel
              </button>
              <input
                type="submit"
                id="submit-button"
                class="cta-button"
                .value=${this.processing?`Submitting...`:`Submit feedback`}
                tabindex="0"
                ?disabled=${this.processing}
              />
            </div>
          </form>
        </div>
      </div>
    `}get upvoteSelected(){return this.vote===`up`}get downvoteSelected(){return this.vote===`down`}upvoteKeypressed(e){(e.key===`Enter`||e.key===` `)&&this.upvoteButtonSelected()}downvoteKeypressed(e){(e.key===`Enter`||e.key===` `)&&this.downvoteButtonSelected()}upvoteButtonSelected(){this.processing||this.voteSubmitted||(this.vote=this.vote===`up`?void 0:`up`,this.handleButtonSelection())}downvoteButtonSelected(){this.processing||this.voteSubmitted||(this.vote=this.vote===`down`?void 0:`down`,this.handleButtonSelection())}async handleButtonSelection(){this.isOpen||(await this.setupRecaptcha(),this.submit())}get chooseVoteErrorClass(){return this.voteNeedsChoosing?`error`:``}get upvoteButtonClass(){switch(this.vote){case`up`:return`selected`;case`down`:return`unselected`;default:return`noselection`}}get downvoteButtonClass(){switch(this.vote){case`up`:return`unselected`;case`down`:return`selected`;default:return`noselection`}}backgroundClicked(e){e.target instanceof Node&&(this.popup?.contains(e.target)||this.cancel(e))}cancel(e){e.preventDefault(),this.closePopup(),this.voteSubmitted||this.resetState()}async submit(e){if(e?.preventDefault(),!this.vote){this.voteNeedsChoosing=!0,this.error=i`Please select a vote.`;return}if(!this.featureIdentifier)throw Error(`featureIdentifier is required`);if(!this.featureFeedbackService)throw Error(`featureFeedbackService is required`);if(!this.recaptchaWidget)throw Error(`recaptchaWidget is required`);let t=this.isOpen;this.processing=!0;try{let e=await this.recaptchaWidget.execute();(await this.featureFeedbackService.submitFeedback({featureIdentifier:this.featureIdentifier,vote:this.vote,comments:this.comments.value,recaptchaToken:e})).success?(this.voteSubmitted=!0,t&&this.closePopup()):this.error=i`There was an error submitting your feedback.`}catch(e){this.error=i`There was an error submitting your feedback.<br />Error:
        ${e instanceof Error?e.message:e}`}this.processing=!1}static get styles(){let e=t`var(--featureFeedbackBlueColor, #194880)`,n=t`var(--featureFeedbackDarkGrayColor, #767676)`,r=t`var(--defaultColorSvgFilter, invert(52%) sepia(0%) saturate(1%) hue-rotate(331deg) brightness(87%) contrast(89%))`,i=t`var(--featureFeedbackBackdropZindex, 5)`,a=t`var(--featureFeedbackModalZindex, 6)`,o=t`var(--featureFeedbackPopupBorderColor, ${e})`,s=t`var(--featureFeedbackSubmitButtonColor, ${e})`,c=t`var(--featureFeedbackBetaButtonBorderColor, ${e})`,l=t`var(--featureFeedbackBetaButtonTextColor, ${e})`,u=t`var(--featureFeedbackBetaButtonBackground, none)`,d=t`var(--featureFeedbackBetaButtonSvgFilter, ${r})`,f=t`var(--featureFeedbackCancelButtonColor, #515151)`,p=t`var(--featureFeedbackPopupBlockerColor, rgba(255, 255, 255, 0.3))`,m=t`var(--featureFeedbackPopupBackgroundColor, #F5F5F7)`,h=t`var(--featureFeedbackPromptFontWeight, bold)`,g=t`var(--featureFeedbackPromptFontSize, 1.4rem)`,_=t`var(--featureFeedbackCommentButtonFontWeight, normal)`,v=t`var(--featureFeedbackCommentButtonFontWeight, 1.4rem)`,y=t`var(--defaultColor, ${n});`,b=t`var(--defaultColorSvgFilter, ${r});`,x=t`var(--upvoteColor, #23765D);`,S=t`var(--upvoteColorSvgFilter, invert(34%) sepia(72%) saturate(357%) hue-rotate(111deg) brightness(97%) contrast(95%));`,C=t`var(--downvoteColor, #720D11);`,w=t`var(--downvoteColorSvgFilter, invert(5%) sepia(81%) saturate(5874%) hue-rotate(352deg) brightness(105%) contrast(95%));`,T=t`var(--unselectedColor, #CCCCCC);`,E=t`var(--unselectedColorSvgFilter, invert(100%) sepia(0%) saturate(107%) hue-rotate(138deg) brightness(89%) contrast(77%));`;return t`
      #container {
        display: inline-block;
      }

      #beta-button {
        font-size: 12px;
        font-weight: bold;
        font-style: italic;
        color: ${l};
        background: ${u};
        border: 1px solid ${c};
        border-radius: 4px;
        padding: 1px 5px;
      }

      .beta-button-thumb svg {
        height: 10px;
        width: 10px;
        filter: ${d};
      }

      .beta-button-thumb.unselected svg {
        filter: ${E};
      }

      #error {
        color: red;
        font-size: 14px;
        text-align: center;
        font-weight: bold;
      }

      #popup-background {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        z-index: ${i};
        background-color: ${p};
        overflow: hidden;
      }

      #popup-background.closed {
        visibility: hidden;
        top: -100%;
        left: -100%;
      }

      #popup {
        position: absolute;
        padding: 10px;
        background-color: ${m};
        border: 1px ${o} solid;
        border-radius: 5px;
        box-shadow: 1px 1px 2px rgba(0, 0, 0, 0.8);
        z-index: ${a};
        max-width: 300px;
        margin-left: 10px;
        margin-right: 10px;
      }

      button,
      input,
      a,
      textarea {
        font-family: inherit;
      }

      button,
      input[type='submit'] {
        background: none;
        cursor: pointer;
        -webkit-appearance: none;
        -moz-appearance: none;
        appearance: none;
        border: none;
      }

      button:disabled,
      input[type='submit']:disabled {
        cursor: default;
        opacity: 0.5;
      }

      #form > div {
        margin-bottom: 10px;
      }

      #form > div:last-child {
        margin-bottom: 0;
      }

      .prompt {
        display: flex;
        align-items: center;
        font-size: ${g};
        font-weight: ${h};
      }

      .prompt > label {
        flex: none;
        cursor: pointer;
      }

      .prompt-text {
        text-align: left;
      }

      #comments {
        width: 100%;
        height: 50px;
        background-color: #ffffff;
        border: 1px #2c2c2c solid;
        border-radius: 4px;
        padding: 7px;
        -webkit-box-sizing: border-box;
        -moz-box-sizing: border-box;
        box-sizing: border-box;
        resize: none;
      }

      #comments::placeholder {
        color: #767676;
      }

      #actions {
        display: flex;
        justify-content: center;
      }

      .cta-button {
        color: white;
        font-size: 14px;
        border-radius: 4px;
        height: 30px;
        margin: 0;
      }

      #cancel-button {
        background-color: ${f};
      }

      #submit-button {
        background-color: ${s};
        margin-left: 10px;
      }

      .vote-button {
        background-color: #ffffff;
        border: 1px solid #767676;
        border-radius: 2px;
        padding: 0;
        width: 25px;
        height: 25px;
        box-sizing: border-box;
        display: flex;
        justify-content: center;
        align-items: center;
        margin-left: 10px;
      }

      .vote-button svg {
        width: 15px;
        height: 15px;
      }

      .vote-button input {
        margin: 0;
        padding: 0;
        -webkit-appearance: none;
        -moz-appearance: none;
        appearance: none;
      }

      .vote-button.noselection {
        border-color: ${y};
      }

      .vote-button.noselection svg {
        filter: ${b};
      }

      .vote-button.unselected {
        border-color: ${T};
      }

      .vote-button.unselected svg {
        filter: ${E};
      }

      .upvote-button.selected {
        border-color: ${x};
      }

      .upvote-button.selected svg {
        filter: ${S};
      }

      .downvote-button.selected {
        border-color: ${C};
      }

      .downvote-button.selected svg {
        filter: ${w};
      }

      .vote-button.error {
        box-shadow: 0 0 4px red;
      }

      form[disabled] .vote-button.unselected {
        cursor: not-allowed;
      }

      #comment-button {
        color: var(--ia-theme-link-color, #4b64ff);
        font-weight: ${_};
        font-size: ${v};
      }
      #comment-button:not([disabled]):hover,
      #comment-button:not([disabled]):active {
        text-decoration: underline;
      }
    `}};m([n({type:String})],_.prototype,`featureIdentifier`,void 0),m([n({type:String})],_.prototype,`prompt`,void 0),m([n({type:String})],_.prototype,`buttonText`,void 0),m([n({type:String})],_.prototype,`displayMode`,void 0),m([n({type:Object})],_.prototype,`recaptchaManager`,void 0),m([n({type:Object})],_.prototype,`resizeObserver`,void 0),m([n({type:Boolean})],_.prototype,`disabled`,void 0),m([n({type:Object})],_.prototype,`featureFeedbackService`,void 0),m([a()],_.prototype,`isOpen`,void 0),m([a()],_.prototype,`processing`,void 0),m([a()],_.prototype,`popupTopX`,void 0),m([a()],_.prototype,`popupTopY`,void 0),m([a()],_.prototype,`vote`,void 0),m([a()],_.prototype,`voteSubmitted`,void 0),m([a()],_.prototype,`error`,void 0),m([a()],_.prototype,`voteNeedsChoosing`,void 0),m([a()],_.prototype,`recaptchaWidget`,void 0),m([f(`#container`)],_.prototype,`container`,void 0),m([f(`#popup`)],_.prototype,`popup`,void 0),m([f(`#comments`)],_.prototype,`comments`,void 0),_=m([d(`feature-feedback`)],_);var v=class{constructor(e){this.serviceUrl=e.serviceUrl}async submitFeedback(e){let t=new URL(this.serviceUrl);t.searchParams.append(`featureId`,e.featureIdentifier),t.searchParams.append(`rating`,e.vote),e.comments&&t.searchParams.append(`comment`,e.comments),t.searchParams.append(`token`,e.recaptchaToken);try{return await(await fetch(t.href)).json()}catch(e){let t;return t=e instanceof Error?e:Error(typeof e==`string`?e:`Unknown error`),{success:!1,error:t}}}async submitSurvey(e){let t=new URL(this.serviceUrl);t.searchParams.append(`surveyId`,e.surveyIdentifier),t.searchParams.append(`token`,e.recaptchaToken);let n=JSON.stringify({surveyResponses:e.responses.map(e=>({name:e.name,...e.rating?{rating:e.rating}:{},...e.comment?{comment:e.comment}:{}}))});try{return await(await fetch(t.href,{method:`POST`,headers:{"Content-Type":`application/json`},body:n})).json()}catch(e){let t;return t=e instanceof Error?e:Error(typeof e==`string`?e:`Unknown error`),{success:!1,error:t}}}},y=t`var(--white, #fff)`,b=t`var(--ia-theme-link-color, #4b64ff)`,x=t`var(--primaryDisableCTAFill, #767676)`,S=t`var(--secondaryCTABorder, #999)`,C=t`var(--primaryCTAFill, #194880)`,w=t`var(--primaryCTAFillRGB, 25, 72, 128)`,T=t`var(--primaryCTABorder, #c5d1df)`,E=t`var(--primaryErrorCTAFill, #d9534f)`,D=t`var(--primaryErrorCTAFillRGB, 229, 28, 38)`,O=t`var(--primaryErrorCTABorder, #d43f3a)`,k=t`var(--secondaryCTAFill, #333)`,A=t`var(--secondaryCTAFillRGB, 51, 51, 51)`,j=t`var(--primaryCTABorder, #979797)`,M=t`var(---primaryWarningFill, #ee8950)`,N=t`var(--primaryWarningFillRGB, 238, 137, 80)`,P=t`
  .ia-button {
    min-height: 3rem;
    cursor: pointer;
    color: ${y};
    line-height: normal;
    border-radius: 0.4rem;
    font-size: 1.4rem;
    font-family: 'Helvetica Neue', Helvetica, Arial, sans-serif;
    border: 1px solid transparent;
    white-space: nowrap;
    appearance: auto;
    box-sizing: border-box;
    display: flex;
    align-items: center;
    transition: all 0.1s ease 0s;
    vertical-align: middle;
    padding: 0 3rem;
    outline-color: ${y};
    outline-offset: -4px;
    user-select: none;
    text-decoration: none;
    width: fit-content;
    -webkit-user-select: none;
    -moz-user-select: none;
    -ms-user-select: none;
    -o-user-select: none;
  }
  .ia-button:focus-visible {
    outline-style: double;
  }
  .ia-button:disabled {
    cursor: not-allowed;
    background-color: ${x};
    border: 1px solid ${S};
  }
  .ia-button.transparent {
    background-color: transparent;
  }

  .ia-button.primary:disabled,
  .ia-button.danger:disabled,
  .ia-button.warning:disabled,
  .ia-button.dark:disabled {
    opacity: 0.5;
  }

  .ia-button.primary {
    background-color: ${C};
    border-color: ${T};
  }
  .ia-button.primary:hover {
    background-color: rgba(${w}, 0.9);
  }
  .ia-button.primary:focus-visible {
    background-color: rgba(${w}, 0.8);
  }
  .ia-button.primary:active {
    background-color: rgba(${w}, 0.7);
  }

  .ia-button.danger {
    background-color: ${E};
    border-color: ${O};
  }
  .ia-button.danger:hover {
    background-color: rgba(${D}, 0.9);
  }
  .ia-button.danger:focus-visible {
    background-color: rgba(${D}, 0.8);
  }
  .ia-button.danger:active {
    background-color: rgba(${D}, 0.7);
  }

  .ia-button.warning {
    background-color: ${M};
    border-color: ${t`var(--primaryWarningBorder, #ec7939)`};
  }
  .ia-button.warning:hover {
    background-color: rgba(${N}, 0.9);
  }
  .ia-button.warning:focus-visible {
    background-color: rgba(${N}, 0.8);
  }
  .ia-button.warning:active {
    background-color: rgba(${N}, 0.7);
  }

  .ia-button.dark {
    background-color: ${k};
    border-color: ${j};
  }
  .ia-button.dark:hover {
    background-color: rgba(${A}, 0.9);
  }
  .ia-button.dark:focus-visible {
    background-color: rgba(${A}, 0.8);
  }
  .ia-button.dark:active {
    background-color: rgba(${A}, 0.7);
  }

  .ia-button.link {
    margin: 0;
    padding: 6px;
    border: 0;
    appearance: none;
    background: none;
    color: ${b};
    text-decoration: none;
    cursor: pointer;
  }
  .ia-button.link:hover {
    text-decoration: underline;
  }
`;t`
  .sr-only {
    position: absolute !important;
    width: 1px !important;
    height: 1px !important;
    margin: -1px !important;
    padding: 0 !important;
    border: 0 !important;
    overflow: hidden !important;
    white-space: nowrap !important;
    clip: rect(1px, 1px, 1px, 1px) !important;
    -webkit-clip-path: inset(50%) !important;
    clip-path: inset(50%) !important;
    user-select: none !important;
  }
`;function F(e){return`disabled`in e&&typeof e.disabled==`boolean`}function I(e){return`numbered`in e&&typeof e.numbered==`boolean`}function L(e){return`validate`in e&&typeof e.validate==`function`}function R(e){return`response`in e&&typeof e.response==`object`&&e.response!==null&&`name`in e.response&&typeof e.response.name==`string`}var z=class extends Error{};async function B(e,t,n=`Operation timed out`){let r=new Promise((e,r)=>setTimeout(r,t,new z(n)));return Promise.race([e,r])}var V,H=V=class extends o{constructor(){super(...arguments),this.buttonText=`Feedback`,this.showButtonThumbs=!1,this.showQuestionNumbers=!1,this.disabled=!1,this.submitTimeout=8e3,this.isOpen=!1,this.submissionState=`idle`,this.popupTopX=0,this.popupTopY=0,this.resizingElement=document.body,this.handleScroll=()=>{this.isOpen&&this.positionPopup()},this.handleEscape=e=>{e.key===`Escape`&&this.cancel(e)}}render(){return i`<div id="container">${this.feedbackButtonTemplate}</div>`}willUpdate(e){if(e.has(`submissionState`)&&(this.isProcessing||this.isSubmitted?this.disableSlottedChildren():this.restoreSlottedChildrenDisabledStates()),e.has(`resizeObserver`)){let t=e.get(`resizeObserver`);this.disconnectResizeObserver(t)}}updated(e){(e.has(`showQuestionNumbers`)||e.has(`assignedElements`))&&this.applyQuestionNumbers();let t=e.get(`submissionState`);t&&(this.submissionState===`error`||t===`error`)&&this.positionPopup(),e.has(`isOpen`)&&this.isOpen&&this.focusFirstFormElement()}disconnectedCallback(){this.removeEscapeListener(),this.stopScrollObserver(),this.disconnectResizeObserver(this.resizeObserver)}disableSlottedChildren(){this.assignedElements.filter(F).forEach(e=>{e.dataset.originallyDisabled=e.disabled?`true`:`false`,e.disabled=!0})}restoreSlottedChildrenDisabledStates(){this.assignedElements.filter(F).forEach(e=>{let{originallyDisabled:t}=e.dataset;if(t===void 0)return;let n=t===`true`;delete e.dataset.originallyDisabled,e.disabled=n})}handleResize(){this.isOpen&&this.positionPopup()}setupEscapeListener(){document.addEventListener(`keyup`,this.handleEscape)}removeEscapeListener(){document.removeEventListener(`keyup`,this.handleEscape)}setupScrollObserver(){document.addEventListener(`scroll`,this.handleScroll)}stopScrollObserver(){document.removeEventListener(`scroll`,this.handleScroll)}setupResizeObserver(){this.resizeObserver&&this.resizeObserver.addObserver({handler:this,target:this.resizingElement})}disconnectResizeObserver(e){(e??this.resizeObserver)?.removeObserver({handler:this,target:this.resizingElement})}getRecaptchaWidget(){if(this.recaptchaWidgetPromise)return this.recaptchaWidgetPromise;if(this.recaptchaManager)return this.recaptchaWidgetPromise=this.recaptchaManager.getRecaptchaWidget(),this.recaptchaWidgetPromise}resetSubmissionState(){this.setSubmissionState(`idle`),this.error=void 0}setSubmissionState(e){this.submissionState!==e&&(this.submissionState=e,this.emitSubmissionStateChanged())}emitSubmissionStateChanged(){this.dispatchEvent(new CustomEvent(`submissionStateChanged`,{detail:this.submissionState}))}get isProcessing(){return this.submissionState===`processing`}get isSubmitted(){return this.submissionState===`submitted`}showPopup(){this.isSubmitted||(this.setupResizeObserver(),this.setupScrollObserver(),this.setupEscapeListener(),this.positionPopup(),this.isOpen=!0,this.getRecaptchaWidget())}closePopup(){this.disconnectResizeObserver(),this.stopScrollObserver(),this.removeEscapeListener(),this.isOpen=!1}positionPopup(){let e=this.container.getBoundingClientRect(),t=this.popup.getBoundingClientRect(),n=window.innerWidth,r=window.innerHeight,i=n/2,a=r/2;e.left<i?this.popupTopX=e.right-20:this.popupTopX=e.left+20-t.width,this.popupTopX+t.width>n&&(this.popupTopX=n-t.width-5),e.top<a?this.popupTopY=e.bottom-10:this.popupTopY=e.top+10-t.height,this.popupTopY+t.height>r&&(this.popupTopY=r-t.height-5),this.popupTopX=Math.max(5,this.popupTopX),this.popupTopY=Math.max(5,this.popupTopY)}get feedbackButtonThumbsTemplate(){return this.showButtonThumbs?i`
      <span class="beta-button-icon">${h}</span>
      <span class="beta-button-icon">${g}</span>
    `:r}get feedbackButtonCheckTemplate(){return i`<span class="beta-button-icon success">&check;</span>`}get feedbackButtonTemplate(){return i`
      <button
        id="beta-button"
        tabindex="0"
        aria-haspopup="dialog"
        ?disabled=${this.disabled}
        @click=${this.showPopup}
      >
        <span id="button-text">${this.buttonText}</span>
        ${this.isSubmitted?this.feedbackButtonCheckTemplate:this.feedbackButtonThumbsTemplate}
      </button>
      ${this.popupTemplate}
    `}get popupTemplate(){let e=this.isProcessing||this.isSubmitted,t=this.isProcessing?V.SUBMIT_BUTTON_PROCESSING_TEXT:V.SUBMIT_BUTTON_NORMAL_TEXT,n=this.error?i`<div id="error">${this.error}</div>`:r,a=c({left:`${this.popupTopX}px`,top:`${this.popupTopY}px`}),o=e=>e.stopPropagation();return i`
      <div
        id="popup-background"
        class=${this.isOpen?`open`:`closed`}
        @click=${this.backgroundClicked}
        @keydown=${this.backgroundClicked}
      >
        <div
          class="focus-trap"
          tabindex="0"
          @focus=${this.focusSubmitButton}
        ></div>
        <div
          id="popup"
          role="dialog"
          aria-modal="true"
          aria-labelledby="survey-heading"
          style=${a}
        >
          <h2 id="survey-heading" class="sr-only">${s(`Feedback Survey`)}</h2>
          <form
            id="form"
            ?disabled=${e}
            @click=${o}
            @keydown=${o}
            @submit=${this.submit}
          >
            <slot id="questions-slot"></slot>
            ${n}
            <div id="actions">
              <button
                type="button"
                id="cancel-button"
                class="cta-button ia-button dark"
                tabindex="0"
                ?disabled=${e}
                @click=${this.cancel}
              >
                ${s(`Cancel`)}
              </button>
              <button
                type="submit"
                id="submit-button"
                class="cta-button ia-button primary"
                tabindex="0"
                ?disabled=${e}
              >
                ${t}
              </button>
            </div>
          </form>
        </div>
        <div
          class="focus-trap"
          tabindex="0"
          @focus=${this.focusFirstFormElement}
        ></div>
      </div>
    `}focusFirstFormElement(){let e=this.getRootNode();for(let t of this.assignedElements)if(t.focus(),e.activeElement===t)return;this.focusCancelButton()}focusCancelButton(){this.cancelButton.focus()}focusSubmitButton(){this.submitButton.focus()}applyQuestionNumbers(){if(!this.showQuestionNumbers){this.assignedElements.forEach(e=>{var t;(t=e.querySelector(`[slot=question-number]`))==null||t.remove()});return}let e=1;this.assignedElements.filter(e=>I(e)&&e.numbered).forEach(t=>{let n=t.querySelector(`[slot=question-number]`);n||(n=document.createElement(`span`),n.setAttribute(`slot`,`question-number`)),n.textContent=`${e}. `,t.append(n),e+=1})}backgroundClicked(e){e.target instanceof Node&&(this.popup?.contains(e.target)||this.cancel(e))}cancel(e){e.preventDefault(),this.closePopup(),this.isSubmitted||this.resetSubmissionState()}validate(){return this.assignedElements.filter(L).map(e=>e.validate()).every(e=>e)}async submit(e){if(e?.preventDefault(),!this.validate()){this.error=i`${V.ERROR_MESSAGE_MISSING_REQUIRED_INPUT}`,this.setSubmissionState(`error`);return}let{surveyIdentifier:t,submitTimeout:n,featureFeedbackService:r}=this;if(this.error=void 0,!t)throw Error(`surveyIdentifier is required`);if(!r)throw Error(`featureFeedbackService is required`);let a=this.getRecaptchaWidget(),o;try{o=await B(a,n)}catch(e){throw Error(`recaptchaWidget load failed: ${e}`)}if(!o)throw Error(`recaptchaWidget is required`);let c=this.isOpen;this.setSubmissionState(`processing`);try{let e=await B(o.execute(),n);(await B(r.submitSurvey({surveyIdentifier:t,responses:this.assignedElements.filter(R).map(e=>e.response),recaptchaToken:e}),n)).success?(this.setSubmissionState(`submitted`),c&&this.closePopup()):(this.error=i`${V.ERROR_MESSAGE_SUBMIT_REQUEST_FAILED}`,this.setSubmissionState(`error`))}catch(e){this.error=i`${V.ERROR_MESSAGE_SUBMIT_REQUEST_FAILED}
        <br />
        ${s(`Error: `)}${e instanceof Error?e.message:e}`,this.setSubmissionState(`error`)}}static get styles(){let e=t`var(--featureFeedbackBlueColor, #194880)`,n=t`var(--defaultColorSvgFilter, invert(52%) sepia(0%) saturate(1%) hue-rotate(331deg) brightness(87%) contrast(89%))`,r=t`var(--featureFeedbackBackdropZindex, 5)`,i=t`var(--featureFeedbackModalZindex, 6)`,a=t`var(--featureFeedbackPopupMaxWidth, 300px)`,o=t`var(--featureFeedbackPopupVerticalPadding, 10px)`,s=t`
      ${o} ${t`var(--featureFeedbackPopupHorizontalPadding, 10px)`}
    `,c=t`var(--featureFeedbackPopupBorderColor, ${e})`,l=t`var(--featureFeedbackBetaButtonBorderColor, ${e})`,u=t`var(--featureFeedbackBetaButtonTextColor, ${e})`,d=t`var(--featureFeedbackBetaButtonBackground, none)`,f=t`var(--featureFeedbackBetaButtonSvgFilter, ${n})`,p=t`var(--featureFeedbackPopupBlockerColor, rgba(255, 255, 255, 0.3))`,m=t`var(--featureFeedbackPopupBackgroundColor, #FBFBFD)`;return[P,t`
      #container {
        display: inline-block;
      }

      #beta-button {
        font-size: 12px;
        font-weight: bold;
        font-style: italic;
        color: ${u};
        border: 1px solid ${l};
        border-radius: 4px;
        padding: 1px 5px;
        background: ${d};
        -webkit-appearance: none;
        -moz-appearance: none;
        appearance: none;
        cursor: pointer;
      }

      .beta-button-icon {
        line-height: 100%;
      }

      .beta-button-icon svg {
        height: 10px;
        width: 10px;
        filter: ${f};
      }

      .beta-button-icon.success {
        filter: ${t`var(--upvoteColorSvgFilter, invert(34%) sepia(72%) saturate(357%) hue-rotate(111deg) brightness(97%) contrast(95%))`};
      }

      #error {
        margin-bottom: 10px;
        color: red;
        font-size: 14px;
        text-align: center;
        font-weight: bold;
      }

      #popup-background {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        z-index: ${r};
        background-color: ${p};
        overflow: hidden;
      }

      #popup-background.closed {
        visibility: hidden;
        top: -100%;
        left: -100%;
      }

      #popup {
        position: absolute;
        max-width: ${a};
        max-height: calc(100vh - 2 * ${o} - 10px);
        padding: ${s};
        border: 1px ${c} solid;
        border-radius: 5px;
        background-color: ${m};
        box-shadow: 1px 1px 2px rgba(0, 0, 0, 0.8);
        z-index: ${i};
        margin-left: 10px;
        margin-right: 10px;
        overflow-y: auto;
        scrollbar-width: thin;
      }

      button,
      input,
      a,
      textarea {
        font-family: inherit;
      }

      #form > div:last-child {
        margin-bottom: 0;
      }

      #actions {
        display: flex;
        justify-content: center;
        align-items: center;
        column-gap: 10px;
      }

      .cta-button {
        color: white;
        font-size: 14px;
        border-radius: 4px;
        height: 30px;
        margin: 0;
      }

      .sr-only,
      .focus-trap {
        position: absolute !important;
        width: 1px !important;
        height: 1px !important;
        margin: -1px !important;
        padding: 0 !important;
        border: 0 !important;
        overflow: hidden !important;
        white-space: nowrap !important;
        clip: rect(1px, 1px, 1px, 1px) !important;
        -webkit-clip-path: inset(50%) !important;
        clip-path: inset(50%) !important;
        user-select: none !important;
      }
    `]}};H.SUBMIT_BUTTON_NORMAL_TEXT=s(`Submit feedback`),H.SUBMIT_BUTTON_PROCESSING_TEXT=s(`Submitting...`),H.ERROR_MESSAGE_MISSING_REQUIRED_INPUT=s(`Please respond to the indicated questions.`),H.ERROR_MESSAGE_SUBMIT_REQUEST_FAILED=s(`There was an error submitting your feedback.`),m([n({type:String})],H.prototype,`surveyIdentifier`,void 0),m([n({type:String})],H.prototype,`buttonText`,void 0),m([n({type:Boolean})],H.prototype,`showButtonThumbs`,void 0),m([n({type:Boolean})],H.prototype,`showQuestionNumbers`,void 0),m([n({type:Boolean})],H.prototype,`disabled`,void 0),m([n({type:Number})],H.prototype,`submitTimeout`,void 0),m([n({type:Object})],H.prototype,`featureFeedbackService`,void 0),m([n({type:Object})],H.prototype,`recaptchaManager`,void 0),m([n({type:Object})],H.prototype,`resizeObserver`,void 0),m([a()],H.prototype,`isOpen`,void 0),m([a()],H.prototype,`submissionState`,void 0),m([a()],H.prototype,`popupTopX`,void 0),m([a()],H.prototype,`popupTopY`,void 0),m([a()],H.prototype,`error`,void 0),m([f(`#container`)],H.prototype,`container`,void 0),m([f(`#popup`)],H.prototype,`popup`,void 0),m([f(`#cancel-button`)],H.prototype,`cancelButton`,void 0),m([f(`#submit-button`)],H.prototype,`submitButton`,void 0),m([p()],H.prototype,`assignedElements`,void 0),H=V=m([d(`ia-feedback-survey`)],H);var U,W=U=class extends o{constructor(){super(),this.prompt=``,this.value=``,this.required=!1,this.disabled=!1,this.skipNumber=!1,this.visible=!0,this.internals=this.attachInternals()}render(){return i`
      <div id="container">
        ${this.promptTextTemplate}${this.commentBoxTemplate}
      </div>
    `}willUpdate(e){e.has(`required`)&&(this.internals.ariaRequired=this.required.toString()),e.has(`disabled`)&&(this.internals.ariaDisabled=this.disabled.toString())}validate(){return!this.required||this.commentBox.value?this.internals.setValidity({}):this.internals.setValidity({valueMissing:!0},`A comment is required.`),this.internals.reportValidity()}get numbered(){return!this.skipNumber}get response(){return{name:this.prompt,comment:this.value}}get promptTextTemplate(){return this.prompt?i`<div id="prompt-text">${this.numbered?i`<slot name="question-number"></slot>`:r}${this.prompt}</div>`:r}get commentBoxTemplate(){let e=this.required?U.DEFAULT_PLACEHOLDER_REQUIRED:U.DEFAULT_PLACEHOLDER_OPTIONAL;return i`
      <textarea
        id="comments"
        tabindex="0"
        placeholder=${this.placeholder??e}
        aria-labelledby="prompt-text"
        aria-required=${this.required}
        .value=${this.value}
        ?disabled=${this.disabled}
        @change=${this.commentChanged}
      ></textarea>
    `}commentChanged(){this.disabled||(this.value=this.commentBox.value,this.value&&this.internals.setValidity({}),this.emitResponseChangedEvent())}emitResponseChangedEvent(){this.dispatchEvent(new CustomEvent(`responseChanged`,{detail:this.response}))}static get styles(){let e=t`var(--commentHeight, 50px)`,n=t`var(--commentResize, none)`,r=t`var(--surveyQuestionMargin, 0 0 15px 0)`,i=t`var(--featureFeedbackPromptFontWeight, bold)`;return t`
      #container {
        margin: ${r};
      }

      #prompt-text {
        text-align: left;
        margin-bottom: 5px;
        flex-grow: 1;
        font-size: ${t`var(--featureFeedbackPromptFontSize, 1.4rem)`};
        font-weight: ${i};
      }

      #comments {
        width: 100%;
        height: ${e};
        background-color: #ffffff;
        border: 1px #2c2c2c solid;
        border-radius: 4px;
        padding: 7px;
        -webkit-box-sizing: border-box;
        -moz-box-sizing: border-box;
        box-sizing: border-box;
        font-family: inherit;
        resize: ${n};
      }

      #comments::placeholder {
        color: #767676;
      }

      :host(:invalid) #comments {
        box-shadow: 0 0 4px red;
      }
    `}};W.DEFAULT_PLACEHOLDER_REQUIRED=s(`Comments`),W.DEFAULT_PLACEHOLDER_OPTIONAL=s(`Comments (optional)`),W.formAssociated=!0,W.shadowRootOptions={...o.shadowRootOptions,delegatesFocus:!0},m([n({type:String})],W.prototype,`prompt`,void 0),m([n({type:String})],W.prototype,`value`,void 0),m([n({type:Boolean,reflect:!0})],W.prototype,`required`,void 0),m([n({type:Boolean,reflect:!0})],W.prototype,`disabled`,void 0),m([n({type:Boolean,reflect:!0})],W.prototype,`skipNumber`,void 0),m([n({type:String})],W.prototype,`placeholder`,void 0),m([f(`#comments`)],W.prototype,`commentBox`,void 0),W=U=m([d(`ia-survey-comment`)],W);var G,K=G=class extends o{constructor(){super(),this.prompt=``,this.vote=void 0,this.required=!1,this.disabled=!1,this.skipNumber=!1,this.showComments=!1,this.visible=!0,this.internals=this.attachInternals()}render(){return i`
      <div id="container">
        <div
          id="prompt-row"
          role="radiogroup"
          aria-labelledby="prompt-text"
          aria-required=${this.required}
          aria-disabled=${this.disabled}
        >
          ${this.promptTextTemplate}${this.voteButtonsTemplate}
        </div>
        ${this.commentFieldTemplate}
      </div>
    `}willUpdate(e){e.has(`required`)&&(this.internals.ariaRequired=this.required.toString()),e.has(`disabled`)&&(this.internals.ariaDisabled=this.disabled.toString())}validate(){return!this.required||this.vote?this.internals.setValidity({}):this.internals.setValidity({valueMissing:!0},`A vote is required.`),this.internals.checkValidity()}get numbered(){return!this.skipNumber}get response(){return{...this.commentBox?.response,name:this.prompt,rating:this.vote}}get promptTextTemplate(){return this.prompt?i`<div id="prompt-text">${this.numbered?i`<slot name="question-number"></slot>`:r}${this.prompt}</div>`:r}get voteButtonsTemplate(){let e=this.vote===`up`,t=this.vote===`down`,n={"vote-button":!0,noselection:this.vote===void 0},r=u({...n,selected:e,unselected:t}),a=u({...n,selected:t,unselected:e});return i`
      <label
        id="upvote"
        class=${r}
        ?disabled=${this.disabled}
      >
        <input
          type="radio"
          name="vote"
          value="up"
          ?checked=${e}
          ?disabled=${this.disabled}
          @click=${this.upvoteButtonSelected}
          @keydown=${this.upvoteKeyPressed}
        />
        ${h}
        <span class="sr-only">${G.UPVOTE_SR_LABEL}</span>
      </label>

      <label
        id="downvote"
        class=${a}
        ?disabled=${this.disabled}
      >
        <input
          type="radio"
          name="vote"
          value="down"
          ?checked=${t}
          ?disabled=${this.disabled}
          @click=${this.downvoteButtonSelected}
          @keydown=${this.downvoteKeyPressed}
        />
        ${g}
        <span class="sr-only">${G.DOWNVOTE_SR_LABEL}</span>
      </label>
    `}get commentFieldTemplate(){return this.showComments?i`
      <ia-survey-comment
        id="comments"
        skipNumber
        .value=${this.comment??``}
        .placeholder=${this.commentPlaceholder}
        ?disabled=${this.disabled}
        @responseChanged=${this.commentChanged}
      ></ia-survey-comment>
    `:r}upvoteKeyPressed(e){(e.key===`Enter`||e.key===` `)&&this.upvoteButtonSelected()}downvoteKeyPressed(e){(e.key===`Enter`||e.key===` `)&&this.downvoteButtonSelected()}upvoteButtonSelected(){this.handleVoteButtonSelection(`up`)}downvoteButtonSelected(){this.handleVoteButtonSelection(`down`)}handleVoteButtonSelection(e){this.disabled||(this.vote=e,this.internals.setValidity({}),this.emitResponseChangedEvent())}commentChanged(e){e.stopPropagation(),!this.disabled&&(this.comment=this.commentBox?.value,this.emitResponseChangedEvent())}emitResponseChangedEvent(){this.dispatchEvent(new CustomEvent(`responseChanged`,{detail:this.response}))}static get styles(){let e=t`var(--surveyQuestionMargin, 0 0 15px 0)`,n=t`var(--featureFeedbackDarkGrayColor, #767676)`,r=t`var(--defaultColorSvgFilter, invert(52%) sepia(0%) saturate(1%) hue-rotate(331deg) brightness(87%) contrast(89%))`,i=t`var(--defaultColor, ${n})`,a=t`var(--defaultColorSvgFilter, ${r})`,o=t`var(--upvoteColor, #23765D)`,s=t`var(--upvoteColorSvgFilter, invert(34%) sepia(72%) saturate(357%) hue-rotate(111deg) brightness(97%) contrast(95%))`,c=t`var(--downvoteColor, #720D11)`,l=t`var(--downvoteColorSvgFilter, invert(5%) sepia(81%) saturate(5874%) hue-rotate(352deg) brightness(105%) contrast(95%))`,u=t`var(--unselectedColor, #CCCCCC)`,d=t`var(--unselectedColorSvgFilter, invert(100%) sepia(0%) saturate(107%) hue-rotate(138deg) brightness(89%) contrast(77%))`,f=t`var(--featureFeedbackPromptFontWeight, bold)`;return t`
      #container {
        margin: ${e};
      }

      #prompt-row {
        display: flex;
        align-items: center;
        margin-bottom: 5px;
      }

      #prompt-text {
        text-align: left;
        flex-grow: 1;
        font-size: ${t`var(--featureFeedbackPromptFontSize, 1.4rem)`};
        font-weight: ${f};
      }

      .vote-button {
        background-color: #ffffff;
        border: 1px solid #767676;
        border-radius: 2px;
        margin-left: 10px;
        padding: 0;
        width: 25px;
        height: 25px;
        box-sizing: border-box;
        display: flex;
        justify-content: center;
        align-items: center;
        flex: none;
        cursor: pointer;
      }

      .vote-button:focus-within {
        outline: 2px solid black;
        outline-offset: 1px;
      }

      .vote-button svg {
        width: 15px;
        height: 15px;
      }

      .vote-button input {
        margin: 0;
        padding: 0;
        -webkit-appearance: none;
        -moz-appearance: none;
        appearance: none;
      }

      .vote-button.noselection {
        border-color: ${i};
      }

      .vote-button.noselection svg {
        filter: ${a};
      }

      .vote-button.unselected {
        border-color: ${u};
      }

      .vote-button.unselected svg {
        filter: ${d};
      }

      #upvote.selected {
        border-color: ${o};
      }

      #upvote.selected svg {
        filter: ${s};
      }

      #downvote.selected {
        border-color: ${c};
      }

      #downvote.selected svg {
        filter: ${l};
      }

      .vote-button.error {
        box-shadow: 0 0 4px red;
      }

      .vote-button.noselection[disabled],
      .vote-button.unselected[disabled] {
        border-color: ${u};
        cursor: not-allowed;
      }

      .vote-button.noselection[disabled] svg,
      .vote-button.unselected[disabled] svg {
        filter: ${d};
      }

      :host(:invalid) #upvote,
      :host(:invalid) #downvote {
        box-shadow: 0 0 4px red;
      }

      #comments {
        --surveyQuestionMargin: 0;
      }

      .sr-only {
        position: absolute !important;
        width: 1px !important;
        height: 1px !important;
        margin: -1px !important;
        padding: 0 !important;
        border: 0 !important;
        overflow: hidden !important;
        white-space: nowrap !important;
        clip: rect(1px, 1px, 1px, 1px) !important;
        -webkit-clip-path: inset(50%) !important;
        clip-path: inset(50%) !important;
        user-select: none !important;
      }
    `}};K.UPVOTE_SR_LABEL=s(`Vote up`),K.DOWNVOTE_SR_LABEL=s(`Vote down`),K.formAssociated=!0,K.shadowRootOptions={...o.shadowRootOptions,delegatesFocus:!0},m([n({type:String})],K.prototype,`prompt`,void 0),m([n({type:String})],K.prototype,`vote`,void 0),m([n({type:String})],K.prototype,`comment`,void 0),m([n({type:String})],K.prototype,`commentPlaceholder`,void 0),m([n({type:Boolean,reflect:!0})],K.prototype,`required`,void 0),m([n({type:Boolean,reflect:!0})],K.prototype,`disabled`,void 0),m([n({type:Boolean,reflect:!0})],K.prototype,`skipNumber`,void 0),m([n({type:Boolean,reflect:!0})],K.prototype,`showComments`,void 0),m([f(`#comments`)],K.prototype,`commentBox`,void 0),K=G=m([d(`ia-survey-vote`)],K);var q=class extends o{constructor(){super(...arguments),this.name=``,this.visible=!1,this.disabled=!1}createRenderRoot(){return this}render(){return r}validate(){return!0}get numbered(){return!1}get response(){return{name:this.name,comment:this.value}}};m([n({type:String,reflect:!0})],q.prototype,`name`,void 0),m([n({type:String,reflect:!0})],q.prototype,`value`,void 0),q=m([d(`ia-survey-extra`)],q);var J=e({FeatureFeedback:()=>_,FeatureFeedbackService:()=>v,IAFeedbackSurvey:()=>H,IASurveyComment:()=>W,IASurveyExtra:()=>q,IASurveyVote:()=>K});export{J as t};