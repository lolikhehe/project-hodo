import{A as e,C as t,O as n,T as r,w as i,x as a}from"./vendor-lit-MQ4DzmMc.js";import"./vendor-topnav-Dcg-INpV.js";import{t as o}from"./index-NFRVgqV2.js";var s=class extends r{constructor(...e){super(...e),this.leftValue=``,this.rightValue=``,this.side=`left`}render(){return n`
      <div id="container">
        <label for="switch-left">${this.leftLabel??this.leftValue}</label>
        <ia-sr-only-text>
          <input
            type="radio"
            id="switch-left"
            name="switch"
            .value=${this.leftValue}
            .checked=${this.side===`left`}
            @change=${this.handleRadioChange}
          />
          <input
            type="radio"
            id="switch-right"
            name="switch"
            .value=${this.rightValue}
            .checked=${this.side===`right`}
            @change=${this.handleRadioChange}
          />
        </ia-sr-only-text>
        <button
          id="switch-button"
          class=${this.side}
          aria-hidden="true"
          @click=${this.handleClick}
        >
          <div id="knob"></div>
        </button>
        <label for="switch-right">${this.rightLabel??this.rightValue}</label>
      </div>
    `}get value(){return this.side===`left`?this.leftValue:this.rightValue}get selectedLabel(){return this.side===`left`?this.leftLabel??this.leftValue:this.rightLabel??this.rightValue}handleClick(){this.side=this.side===`left`?`right`:`left`,this.emitChangeEvent()}handleRadioChange(){this.side=this.leftRadio.checked?`left`:`right`,this.emitChangeEvent()}emitChangeEvent(){let e=new CustomEvent(`change`,{detail:this.value});this.dispatchEvent(e)}static get styles(){return e`
      :host {
        --switchWidth--: var(--switchWidth, 30px);
        --switchHeight--: var(--switchHeight, 14px);
        --switchMarginLeft--: var(--switchMarginLeft, 5px);
        --switchMarginRight--: var(--switchMarginRight, 5px);
        --switchBorderWidth--: var(--switchBorderWidth, 3px);
        --switchBgColor--: var(--switchBgColor, #194880);
        --switchBorderColor--: var(--switchBorderColor, #194880);
        --labelFontSize--: var(--labelFontSize, 1.3rem);
        --knobColor--: var(--knobColor, white);
        --knobTransitionDuration--: var(--knobTransitionDuration, 0.25s);
        --knobTransitionFn--: var(--knobTransitionFn, ease);
      }

      #container {
        display: inline-flex;
        align-items: center;
        flex-wrap: nowrap;
        font-size: var(--labelFontSize--);
      }

      #switch-button {
        width: var(--switchWidth--);
        height: var(--switchHeight--);
        margin: 0 var(--switchMarginRight--) 0 var(--switchMarginLeft--);
        padding: 0;
        border: var(--switchBorderWidth--) solid var(--switchBorderColor--);
        border-radius: var(--switchHeight--);
        box-sizing: content-box;
        background: var(--switchBgColor--);
        appearance: none;
        cursor: pointer;
      }

      #switch-button.left #knob {
        transform: translateX(0);
      }

      #switch-button.right #knob {
        transform: translateX(
          calc(var(--switchWidth--) - var(--switchHeight--))
        );
      }

      #switch-button:focus-visible {
        outline: 2px solid black;
        outline-offset: 2px;
      }

      #knob {
        display: block;
        width: var(--switchHeight--);
        height: var(--switchHeight--);
        border-radius: 50%;
        background: var(--knobColor--);
        transition: transform var(--knobTransitionDuration--)
          var(--knobTransitionFn--);
      }

      @media (prefers-reduced-motion: reduce) {
        #knob {
          transition-duration: 0.001s !important; /* Imperceptibly fast */
        }
      }
    `}};o([t({type:String,attribute:!0})],s.prototype,`leftValue`,void 0),o([t({type:String,attribute:!0})],s.prototype,`leftLabel`,void 0),o([t({type:String,attribute:!0})],s.prototype,`rightValue`,void 0),o([t({type:String,attribute:!0})],s.prototype,`rightLabel`,void 0),o([t({type:String,attribute:!0})],s.prototype,`side`,void 0),o([a(`#switch-left`)],s.prototype,`leftRadio`,void 0),s=o([i(`iaux-toggle-switch`)],s);