import{A as e,C as t,F as n,N as r,O as i,P as a,T as o,_ as s,k as c,x as l,y as u}from"./vendor-lit-MQ4DzmMc.js";import{s as d}from"./vendor-shared-62wh2Xkx.js";import{r as f,t as p}from"./vendor-topnav-Dcg-INpV.js";var m=c`
<svg class="caret-up-svg" viewBox="0 0 8 4" xmlns="http://www.w3.org/2000/svg">
<path d="m6.7226499 3.51689722c.22976435.15317623.54019902.0910893.69337525-.13867505.13615665-.20423497.10222882-.47220946-.06836249-.63681849l-.07031256-.05655675-3.2773501-2.18490007-3.2773501 2.18490007c-.22976434.15317623-.29185128.4636109-.13867505.69337524.13615665.20423498.39656688.27598409.61412572.18182636l.07924953-.04315131 2.7226499-1.81402514z"
  fill=""></path>
</svg>
`,h=c`
<svg class="caret-down-svg" viewBox="0 0 8 4" xmlns="http://www.w3.org/2000/svg">
<path d="m6.7226499.58397485c.22976435-.15317623.54019902-.09108929.69337525.13867505.13615665.20423498.10222882.47220947-.06836249.63681849l-.07031256.05655676-3.2773501 2.18490006-3.2773501-2.18490006c-.22976434-.15317623-.29185128-.4636109-.13867505-.69337525.13615665-.20423497.39656688-.27598409.61412572-.18182636l.07924953.04315131 2.7226499 1.81402515z"
fill=""></path>
</svg>
`,g=class extends o{constructor(){super(...arguments),this.open=!1,this.isDisabled=!1,this.displayCaret=!1,this.closeOnSelect=!1,this.openViaButton=!0,this.usePopover=!1,this.includeSelectedOption=!1,this.selectedOption=``,this.options=[],this.optionSelected=()=>{},this.isCustomList=!1,this.hasCustomClickHandler=!1,this.closeOnEscape=!1,this.closeOnBackdropClick=!1,this.boundKeyboardListener=e=>{switch(e.key){case`Escape`:case`Esc`:this.closeOptions();break;default:break}},this.closeOptions=e=>{e&&e.type===`click`&&e.stopPropagation(),this.open=!1,this.updatePopoverState()}}async firstUpdated(){await new Promise(e=>{setTimeout(e,0)}),this.addEventListener(`closeDropdown`,this.closeOptions)}willUpdate(e){e.has(`open`)&&this.updatePopoverState()}disconnectedCallback(){super.disconnectedCallback?.(),this.removeKeyboardListener()}setupKeyboardListener(){this.closeOnEscape&&document.addEventListener(`keydown`,this.boundKeyboardListener)}removeKeyboardListener(){this.closeOnEscape&&document.removeEventListener(`keydown`,this.boundKeyboardListener)}get dropdownState(){return this.open?(this.setupKeyboardListener(),`open`):(this.removeKeyboardListener(),`closed`)}toggleOptions(){this.open=!this.open,this.updatePopoverState()}updatePopoverState(){this.usePopover&&(this.dropdownMenu?.togglePopover?.(this.open),this.open&&this.positionDropdownMenu())}positionDropdownMenu(){if(!this.dropdownMenu)return;let e=this.container.getBoundingClientRect();this.dropdownMenu.style.left=`${e.left}px`,this.dropdownMenu.style.top=`${e.bottom}px`,this.dropdownMenu.style.minWidth=`${e.width}px`}mainButtonClicked(){this.openViaButton?this.toggleOptions():this.mainButtonLabelSlotted[0]?.click()}mainButtonKeyDown(e){(e.key===`Enter`||e.key===` `)&&(this.mainButtonClicked(),e.preventDefault())}caretKeyDown(e){(e.key===`Enter`||e.key===` `)&&(this.toggleOptions(),e.preventDefault())}renderOption(e){let{label:t,url:n=void 0,id:r}=e,a,o=this.selectedOption===r?`selected`:``;return a=n?i`<a
        href=${n}
        @click=${t=>this.optionClicked(t,e)}
        >${t}</a
      >`:i`<button
        @click=${t=>this.optionClicked(t,e)}
      >
        ${t}
      </button>`,i`<li role="menuitem" class=${o}>${a}</li>`}optionClicked(e,t){e.stopPropagation(),this.selectedOption!==t.id&&(this.selectedOption=t.id,this.dispatchEvent(new CustomEvent(`optionSelected`,{detail:{option:t}})),t.selectedHandler?.(t)),this.closeOnSelect&&(this.closeOptions(),this.mainButton.focus())}get availableOptions(){return this.includeSelectedOption?this.options:this.options.filter(e=>this.selectedOption!==e.id)}get caretUpTemplate(){return i`
      <span ?hidden=${!this.open} class="caret-up">
        <slot name="caret-up">${m}</slot>
      </span>
    `}get caretDownTemplate(){return i`
      <span ?hidden=${this.open} class="caret-down">
        <slot name="caret-down">${h}</slot>
      </span>
    `}get caretTemplate(){return this.displayCaret?this.openViaButton?i`
        <span class="caret" aria-hidden="true">
          ${this.caretUpTemplate} ${this.caretDownTemplate}
        </span>
      `:i`
      <button
        class="caret"
        aria-labelledby="caret-label"
        aria-haspopup="true"
        aria-expanded=${this.open}
        @click=${s(this.shouldAttachEventHandlers,()=>this.toggleOptions)}
        @keydown=${s(this.shouldAttachEventHandlers,()=>this.caretKeyDown)}
        ?disabled=${this.isDisabled}
      >
        ${this.caretUpTemplate} ${this.caretDownTemplate}
      </button>
    `:i``}get dropdownTemplate(){return this.isCustomList?i`<slot name="list"></slot>`:i`${this.availableOptions.map(e=>this.renderOption(e))}`}get backdropTemplate(){return!this.closeOnBackdropClick||!this.open?i``:i`
      <div
        id="dropdown-backdrop"
        @keyup=${this.closeOptions}
        @click=${this.closeOptions}
      ></div>
    `}get shouldNestCaretInButton(){return this.openViaButton}get shouldAttachEventHandlers(){return!this.isDisabled&&!this.hasCustomClickHandler}get toggleLabel(){return this.optionGroup?a(n`Toggle ${this.optionGroup}`):a(`Toggle options`)}render(){return i`
      <div class="ia-dropdown-group ${this.open?`open`:``}">
        <div class="button-row">
          <button
            class="click-main"
            aria-haspopup=${this.openViaButton}
            aria-expanded=${this.open}
            @click=${s(this.shouldAttachEventHandlers,()=>this.mainButtonClicked)}
            @keydown=${s(this.shouldAttachEventHandlers,()=>this.mainButtonKeyDown)}
            ?disabled=${this.isDisabled}
          >
            <span class="sr-only" id="caret-label">${this.toggleLabel}</span>
            <slot name="dropdown-label"></slot>
            ${s(this.shouldNestCaretInButton,()=>this.caretTemplate)}
          </button>
          ${s(!this.shouldNestCaretInButton,()=>this.caretTemplate)}
        </div>

        <ul
          id="dropdown-main"
          class=${this.dropdownState}
          role="menu"
          ?popover=${this.usePopover}
        >
          ${this.dropdownTemplate}
        </ul>

        ${this.backdropTemplate}
      </div>
    `}static get styles(){let t=e`var(--dropdownBorderWidth, 1px)`,n=e`var(--dropdownBorderRadius, 4px)`,r=e`var(--dropdownBorderColor, #fff)`,i=e`var(--dropdownBgColor, #333)`,a=e`var(--dropdownTextColor, #fff)`,o=e`var(--dropdownHoverBgColor, rgba(255, 255, 255, 0.3))`,s=e`var(--dropdownSelectedBgColor, #fff)`,c=e`var(--dropdownMainButtonBgColor, transparent)`,l=e`var(--dropdownTextAlign, inherit)`,u=e`var(--dropdownBackdropZIndex, 1)`,d=e`var(--dropdownListZIndex, 2)`;return[p,e`
        :host {
          display: inline;
          color: ${a};
        }

        svg.caret-up-svg,
        svg.caret-down-svg,
        ::slotted(svg.caret-up-svg),
        ::slotted(svg.caret-down-svg) {
          fill: var(--dropdownCaretColor, #fff);
          vertical-align: middle;
        }

        .button-row {
          display: flex;
          height: 100%;
        }

        button.click-main {
          background: ${c};
          color: inherit;
          padding: var(--dropdownMainButtonPadding, 0px);
          border: var(--dropdownMainButtonBorder, none);
          border-radius: var(--dropdownMainButtonBorderRadius, none);
          cursor: pointer;
          display: flex;
          align-items: center;
          justify-content: center;
          align-content: center;
          flex-wrap: nowrap;
          flex-direction: var(--dropdownMainButtonFlexDirection, row);
        }

        .open button.click-main {
          /* When the dropdown is open, give the buttom the same z-index
           as the dropdown menu, so that it remains clickable despite
           the backdrop. */
          z-index: ${d};
        }

        button.click-main:disabled {
          pointer-events: none;
          cursor: not-allowed;
          opacity: 0.5;
          /* Disable text selection on disabled button */
          -webkit-user-select: none; /* Safari */
          -ms-user-select: none; /* IE 10 and IE 11 */
          user-select: none; /* Standard syntax */
        }

        button.click-main:hover {
          background-color: var(
            --dropdownMainButtonHoverBgColor,
            ${c}
          );
        }

        button.click-main:focus,
        button.click-main:focus-visible {
          background-color: var(
            --dropdownMainButtonFocusBgColor,
            ${c}
          );
        }

        button.click-main:active {
          background-color: var(
            --dropdownMainButtonActiveBgColor,
            ${c}
          );
        }

        button slot[name='dropdown-label'] {
          /* Set var to 0px for column layout */
          padding-right: var(--buttonSlotPaddingRight, 5px);
          display: inline-block;
        }

        .ia-dropdown-group {
          width: inherit;
          height: inherit;
          position: relative;
        }

        .sr-only {
          border: 0 !important;
          clip: rect(1px, 1px, 1px, 1px) !important;
          -webkit-clip-path: inset(50%) !important;
          clip-path: inset(50%) !important;
          height: 1px !important;
          margin: -1px !important;
          overflow: hidden !important;
          padding: 0 !important;
          position: absolute !important;
          width: 1px !important;
          white-space: nowrap !important;
          -webkit-user-select: none !important;
          user-select: none !important;
        }

        .caret {
          /* Maintain centered caret position but with a full-height clickable region */
          display: flex;
          align-self: stretch;
          align-items: center;
          padding: var(--caretPadding, 0px);
        }

        button.caret {
          appearance: none;
          background: none;
          border: none;
          cursor: pointer;
        }

        .caret svg {
          height: var(--caretHeight, 10px);
          width: var(--caretWidth, 20px);
        }

        #dropdown-backdrop {
          position: fixed;
          top: 0;
          left: 0;
          width: 100vw;
          height: 100vh;
          background-color: transparent;
          z-index: ${u};
        }

        ul {
          z-index: ${d};
        }

        #dropdown-main.closed {
          visibility: hidden;
          height: 1px;
          width: 1px;
        }

        #dropdown-main {
          position: var(--dropdownListPosition, absolute);
          list-style: none;
          margin: var(--dropdownOffsetTop, 5px) 0 0 0;
          padding: var(--dropdownPadding, 0);
          color: ${a};
          background: ${i};

          font-size: var(--dropdownFontSize, inherit);

          border-top: var(--dropdownBorderTopWidth, ${t});
          border-right: var(--dropdownBorderRightWidth, ${t});
          border-bottom: var(
            --dropdownBorderBottomWidth,
            ${t}
          );
          border-left: var(--dropdownBorderLeftWidth, ${t});
          /* Must be after border-width settings for specificity */
          border-style: solid;
          border-color: ${r};

          border-radius: var(
              --dropdownBorderTopLeftRadius,
              ${n}
            )
            var(--dropdownBorderTopRightRadius, ${n})
            var(--dropdownBorderBottomRightRadius, ${n})
            var(--dropdownBorderBottomLeftRadius, ${n});

          white-space: var(--dropdownWhiteSpace, normal);

          /* Prevent top/bottom inner li from overlapping inner border */
          overflow: hidden;
        }

        #dropdown-main li:hover {
          background-color: ${o};
          color: var(--dropdownHoverTextColor, #fff);
          list-style: none;
          cursor: pointer;
        }

        #dropdown-main li:hover:first-child {
          border-top-color: ${o};
        }

        ul#dropdown-main li:hover:last-child {
          border-bottom-color: ${o};
        }

        #dropdown-main li:hover:not(:first-child) {
          border-top: 0.5px solid var(--dropdownHoverTopBottomBorderColor, #333);
        }
        #dropdown-main li:hover:not(:last-child) {
          border-bottom: 0.5px solid
            var(--dropdownHoverTopBottomBorderColor, #333);
        }

        #dropdown-main li.selected:last-child {
          border-bottom-color: ${s};
        }

        #dropdown-main li.selected:first-child {
          border-top-color: ${s};
        }

        #dropdown-main li:hover > *,
        #dropdown-main li:focus-within > * {
          background-color: ${o};
          color: var(--dropdownHoverTextColor, #fff);
        }

        #dropdown-main li.selected > * {
          background-color: ${s};
          color: var(--dropdownSelectedTextColor, #2c2c2c);
        }

        #dropdown-main li {
          background: ${i};
          list-style: none;
          height: 30px;
          cursor: pointer;
          border-bottom: 0.5px solid ${i};
          border-top: 0.5px solid ${i};
        }

        #dropdown-main li button {
          background: none;
          color: inherit;
          border: none;
          font: inherit;
          cursor: pointer;
          outline: inherit;
        }

        #dropdown-main li a {
          text-decoration: none;
          display: block;
          box-sizing: border-box;
        }

        #dropdown-main li:first-child {
          border-top-left-radius: var(--dropdownBorderTopLeftRadius, 4px);
          border-top-right-radius: var(--dropdownBorderTopRightRadius, 4px);
        }

        #dropdown-main li:last-child {
          border-bottom-right-radius: var(
            --dropdownBorderBottomRightRadius,
            4px
          );
          border-bottom-left-radius: var(--dropdownBorderBottomLeftRadius, 4px);
        }

        /* cover the list with the label */
        #dropdown-main li > * > :first-child {
          margin: 0;
          display: flex;
          align-items: center;
          justify-content: flex-start;
          align-content: center;
          flex-wrap: nowrap;
          height: 100%;
          padding: var(--dropdownItemPaddingTop, 5px)
            var(--dropdownItemPaddingRight, 10px)
            var(--dropdownItemPaddingBottom, 5px)
            var(--dropdownItemPaddingLeft, 10px);
          box-sizing: border-box;
        }

        #dropdown-main li > * {
          width: 100%;
          height: inherit;
          color: ${a};
          background: var(--dropdownItemButtonBgColor, transparent);
          padding: var(--dropdownItemButtonPadding, 0);
          text-align: ${l};
        }
      `]}};d([t({type:Boolean,reflect:!0})],g.prototype,`open`,void 0),d([t({type:Boolean,reflect:!0})],g.prototype,`isDisabled`,void 0),d([t({type:Boolean})],g.prototype,`displayCaret`,void 0),d([t({type:Boolean})],g.prototype,`closeOnSelect`,void 0),d([t({type:Boolean})],g.prototype,`openViaButton`,void 0),d([t({type:Boolean})],g.prototype,`usePopover`,void 0),d([t({type:Boolean})],g.prototype,`includeSelectedOption`,void 0),d([t({type:String})],g.prototype,`selectedOption`,void 0),d([t({attribute:!1})],g.prototype,`options`,void 0),d([t({type:String})],g.prototype,`optionGroup`,void 0),d([t({attribute:!1})],g.prototype,`optionSelected`,void 0),d([t({type:Boolean,reflect:!0})],g.prototype,`isCustomList`,void 0),d([t({type:Boolean,reflect:!0})],g.prototype,`hasCustomClickHandler`,void 0),d([t({type:Boolean,reflect:!0})],g.prototype,`closeOnEscape`,void 0),d([t({type:Boolean,reflect:!0})],g.prototype,`closeOnBackdropClick`,void 0),d([l(`.ia-dropdown-group`)],g.prototype,`container`,void 0),d([l(`#dropdown-main`)],g.prototype,`dropdownMenu`,void 0),d([l(`.click-main`)],g.prototype,`mainButton`,void 0),d([u({slot:`dropdown-label`})],g.prototype,`mainButtonLabelSlotted`,void 0),g=d([f(`ia-dropdown`),r()],g);