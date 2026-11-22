import{A as e,C as t,E as n,O as r,T as i,k as a,x as o}from"./vendor-lit-MQ4DzmMc.js";import{s}from"./vendor-shared-62wh2Xkx.js";import{r as c}from"./vendor-topnav-Dcg-INpV.js";var l=a`
  <svg viewBox="0 0 40 40" version="1.1" xmlns="http://www.w3.org/2000/svg">
    <path
      class="fill-color"
      fill-rule="evenodd"
      d="m29.1923882 10.8076118c.5857864.5857865.5857864 1.535534 0 2.1213204l-7.0711162 7.0703398 7.0711162 7.0717958c.5857864.5857864.5857864 1.5355339 0 2.1213204-.5857865.5857864-1.535534.5857864-2.1213204 0l-7.0717958-7.0711162-7.0703398 7.0711162c-.5857864.5857864-1.5355339.5857864-2.1213204 0-.5857864-.5857865-.5857864-1.535534 0-2.1213204l7.0706602-7.0717958-7.0706602-7.0703398c-.5857864-.5857864-.5857864-1.5355339 0-2.1213204.5857865-.5857864 1.535534-.5857864 2.1213204 0l7.0703398 7.0706602 7.0717958-7.0706602c.5857864-.5857864 1.5355339-.5857864 2.1213204 0z"
    />
  </svg>
`,u=class extends i{constructor(){super(...arguments),this.value=``,this.clearButtonScreenReaderLabel=`Clear`,this.focusOnClear=!0,this.forceClearButton=!1}render(){let e=!this.value&&!this.forceClearButton;return r`
      <div id="container">
        <slot name="icon"></slot>
        <label for="text-input" class="sr-only"
          >${this.screenReaderLabel??n}</label
        >
        <input
          id="text-input"
          type="text"
          inputmode="search"
          enterkeyhint="search"
          autocapitalize="off"
          placeholder=${this.placeholder??n}
          .value=${this.value??n}
          aria-controls=${this.ariaControls??n}
          @input=${this.onTextInput}
          @keypress=${this.onKeyPress}
        />
        <button
          id="clear-button"
          type="button"
          ?hidden=${e}
          @click=${this.clearButtonClicked}
        >
          <span class="clear-icon" aria-hidden="true">${l}</span>
          <span class="sr-only">${this.clearButtonScreenReaderLabel}</span>
        </button>
      </div>
    `}onTextInput(){this.value=this.textInput.value}onKeyPress(e){if(e.key===`Enter`){this.textInput.blur();let e=new CustomEvent(`submit`,{detail:this.value});this.dispatchEvent(e)}}clearButtonClicked(){let e=this.textInput.value;this.value=``,this.focusOnClear&&this.textInput.focus();let t=new CustomEvent(`clear`,{detail:e});this.dispatchEvent(t);let n=new InputEvent(`input`,{inputType:`deleteContentBackward`});this.dispatchEvent(n)}};u.shadowRootOptions={...i.shadowRootOptions,delegatesFocus:!0},u.styles=e`
    :host {
      --input-height: 3rem;
      --clear-button-top: 0;
      --clear-button-right: 2px;
    }

    #container {
      position: relative;
      display: flex;
      align-items: center;
      width: 100%;
      height: var(--input-height);
    }

    #text-input {
      width: 100%;
      height: 100%;
      padding: var(--input-padding, 0 1rem);
      padding-inline-end: var(--input-height);
      border: var(--input-border-width, 1px) var(--input-border-style, solid)
        var(--input-border-color, #ccc);
      border-radius: var(--input-border-radius, 2rem);
      background-image: var(--input-background-image, none);
      background-color: var(--input-background-color, transparent);
      color: var(--input-color, #555);
      font-size: var(--input-font-size, 1.7rem);
      line-height: var(--input-line-height, 1.5);
      box-shadow: var(--input-box-shadow, inset 0 1px 1px rgba(0, 0, 0, 0.075));
      -webkit-box-shadow: var(
        --input-box-shadow,
        inset 0 1px 1px rgba(0, 0, 0, 0.075)
      );
    }

    #text-input:focus {
      border-color: var(--input-focused-border-color, #66afe9);
      outline: 0;
      box-shadow: var(
        --input-focused-box-shadow,
        inset 0 1px 1px rgb(0 0 0 / 8%),
        0 0 8px rgb(102 175 233 / 60%)
      );
      -webkit-box-shadow: var(
        --input-focused-box-shadow,
        inset 0 1px 1px rgb(0 0 0 / 8%),
        0 0 8px rgb(102 175 233 / 60%)
      );
    }

    #clear-button {
      position: absolute;
      display: flex;
      justify-content: center;
      align-items: center;
      inset-block-start: var(--clear-button-top);
      inset-inline-end: var(--clear-button-right);
      height: var(--clear-button-height, var(--input-height));
      width: var(--clear-button-width, var(--input-height));
      padding: var(--clear-button-padding, 4px);
      border: var(--clear-button-border, 0);
      background: none;
      appearance: none;
      cursor: pointer;
    }

    #clear-button[hidden] {
      display: none;
    }

    .clear-icon {
      display: block;
      width: 100%;
      height: 100%;
      background: var(--clear-button-icon-background, #2c2c2c);
      border-radius: 50%;
      pointer-events: none;
    }

    .clear-icon svg {
      display: block;
      width: 100%;
      height: 100%;
    }

    .clear-icon .fill-color {
      fill: var(--clear-button-icon-color, #fff);
    }

    /* Fallback support for older browsers without newer bidirectional rules */
    @supports not ((inset-block-start: 0) and (padding-inline-end: 0)) {
      #text-input {
        padding-right: var(--input-height);
      }

      #clear-button {
        top: var(--clear-button-top);
        right: var(--clear-button-right);
      }
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
    }
  `,s([t({type:String})],u.prototype,`value`,void 0),s([t({type:String})],u.prototype,`placeholder`,void 0),s([t({type:String})],u.prototype,`screenReaderLabel`,void 0),s([t({type:String})],u.prototype,`clearButtonScreenReaderLabel`,void 0),s([t({type:String})],u.prototype,`ariaControls`,void 0),s([t({type:Boolean})],u.prototype,`focusOnClear`,void 0),s([t({type:Boolean,reflect:!0})],u.prototype,`forceClearButton`,void 0),s([o(`#text-input`)],u.prototype,`textInput`,void 0),u=s([c(`ia-clearable-text-input`)],u);export{u as t};