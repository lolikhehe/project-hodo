import{A as e,C as t,E as n,N as r,O as i,P as a,S as o,T as s,k as c,m as l,o as u,s as d,w as f,x as p}from"./vendor-lit-MQ4DzmMc.js";import{g as m,t as h}from"./index-NFRVgqV2.js";import{LOCALIZATION_AB_TEST_NAME as g}from"./localization-experiment-BXq-ONhb.js";var _={[m.en]:`English`,[m.es]:`Español`},v={[m.en]:`en-US`,[m.es]:`es-ES`},y=c`
  <svg viewBox="0 0 24 24" width="16" height="16" fill="none"
    stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
    <circle cx="12" cy="12" r="9.25" />
    <path d="M2.75 12h18.5" />
    <path d="M12 2.75c2.4 2.5 3.6 5.6 3.6 9.25s-1.2 6.75-3.6 9.25" />
    <path d="M12 2.75c-2.4 2.5-3.6 5.6-3.6 9.25s1.2 6.75 3.6 9.25" />
  </svg>
`,b=class extends s{constructor(...e){super(...e),this.pickerVisible=!1,this.pointerFocused=!1,this.setupRunId=0}willUpdate(e){e.has(`appServices`)&&(this.setupComplete=this.setup()),this.toggleAttribute(`hidden`,!this.pickerVisible)}render(){if(!this.pickerVisible||!this.localizationManager)return n;let e=this.localizationManager.language;return i`
      <div
        class=${l({container:!0,pointerFocused:this.pointerFocused})}
      >
        <label for="language-select" class="sr-only">${a(`Language`)}</label>
        <span class="globe" aria-hidden="true">${y}</span>
        <span class="code" aria-hidden="true">
          ${e.toUpperCase()}
        </span>
        <select
          id="language-select"
          title=${a(`Choose a language`)}
          @pointerdown=${this.pointerDown}
          @keydown=${this.keyDown}
          @blur=${this.blurred}
          @change=${this.languageSelected}
        >
          ${d(Object.values(m),t=>i`
              <option
                value=${t}
                lang=${t}
                .selected=${u(t===e)}
              >
                ${_[t]} (${v[t]})
              </option>
            `)}
        </select>
      </div>
    `}pointerDown(){this.pointerFocused=!0}keyDown(){this.pointerFocused=!1}blurred(){this.pointerFocused=!1}async setup(){this.pickerVisible=!1,this.abVariant=void 0;let e=++this.setupRunId,t=()=>e===this.setupRunId;try{let e=await this.appServices?.get();if(!e||!t())return;let n=await e.localizationExperiment.get(),r=await n.variant();if(!t()||(this.abVariant=r,r!==`On`)||(await n.restoreLanguage(),!t()))return;this.localizationManager=await e.localizationManager.get();try{this.analyticsHandler=await e.analyticsHandler.get()}catch(e){console.error(`Error resolving the language-picker analytics handler`,e)}if(!t())return;this.pickerVisible=!0;try{this.trackRestoredLanguage()}catch(e){console.error(`Error sending language-picker analytics`,e)}}catch(e){console.error(`Error setting up the language picker`,e),t()&&(this.pickerVisible=!1)}}trackRestoredLanguage(){let e=this.localizationManager?.language;!e||e===m.en||this.analyticsHandler?.sendEventNoSampling({category:g,action:`LanguageKept`,label:`Language:${e}|Variant:${this.abVariant}`})}async languageSelected(){let e=this.select?.value,t=Object.values(m).find(t=>t===e);if(!t||!this.localizationManager)return;let n=this.localizationManager.language;try{await this.localizationManager.setLanguage(t,!0),this.analyticsHandler?.sendEventNoSampling({category:g,action:`LanguageChanged`,label:`From:${n}|To:${t}|Variant:${this.abVariant}`})}catch(e){console.error(`Error switching language`,e),this.requestUpdate()}}static get styles(){return e`
      :host {
        --iconColor--: var(--languagePickerIconColor, var(--iconFill, white));
        --iconHoverColor--: var(
          --languagePickerIconHoverColor,
          var(--linkHoverColor, white)
        );
        --focusOutlineColor--: var(--languagePickerFocusOutlineColor, white);
        --size--: var(
          --languagePickerSize,
          calc(40 * var(--topnavUnit--, 1px))
        );
        --iconSize--: var(
          --languagePickerIconSize,
          calc(24 * var(--topnavUnit--, 1px))
        );

        --justify--: var(--languagePickerJustify, center);
        --padding--: var(
          --languagePickerPadding,
          0 calc(4 * var(--topnavUnit--, 1px))
        );

        display: inline-flex;
        /* WebKit keeps an inherited visibility:hidden from the app's loading gate after it lifts. */
        visibility: visible;
      }

      :host([hidden]) {
        display: none !important;
      }

      .container {
        position: relative;
        display: flex;
        align-items: center;
        justify-content: var(--justify--);
        gap: calc(4 * var(--topnavUnit--, 1px));
        box-sizing: border-box;
        min-width: var(--size--);
        height: 100%;
        min-height: var(--size--);
        padding: var(--padding--);
        color: var(--iconColor--);
      }

      .container:hover,
      .container:focus-within {
        color: var(--iconHoverColor--);
      }

      .container:not(.pointerFocused):has(select:focus-visible) {
        outline: 2px solid var(--focusOutlineColor--);
        outline-offset: -2px;
      }

      .globe {
        display: inline-flex;
        pointer-events: none;
      }

      .code {
        font-size: calc(14 * var(--topnavUnit--, 1px));
        line-height: 1;
        pointer-events: none;
      }

      .globe svg {
        width: var(--iconSize--);
        height: var(--iconSize--);
      }

      select {
        position: absolute;
        inset: 0;
        width: 100%;
        height: 100%;
        margin: 0;
        padding: 0;
        border: 0;
        opacity: 0;
        cursor: pointer;
        appearance: none;
        font-size: 16px;
      }

      .sr-only {
        position: absolute;
        width: 1px;
        height: 1px;
        padding: 0;
        margin: -1px;
        overflow: hidden;
        clip: rect(0, 0, 0, 0);
        white-space: nowrap;
        border: 0;
      }
    `}};h([t({type:Object})],b.prototype,`appServices`,void 0),h([o()],b.prototype,`pickerVisible`,void 0),h([o()],b.prototype,`localizationManager`,void 0),h([o()],b.prototype,`pointerFocused`,void 0),h([p(`select`)],b.prototype,`select`,void 0),b=h([f(`language-picker`),r()],b);export{b as LanguagePicker};