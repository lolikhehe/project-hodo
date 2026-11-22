import{A as e,C as t,N as n,O as r,P as i,S as a,T as o,_ as s,v as c,w as l,x as u}from"./vendor-lit-MQ4DzmMc.js";import{t as d}from"./index-NFRVgqV2.js";var f=class extends o{constructor(...e){super(...e),this.titleSrOnly=!1,this.numCarouselItems=0,this.navPreviousVisible=!1,this.navNextVisible=!1,this.showNavBtns=!0}render(){return r`
      ${s(this.carouselTitle,()=>this.carouselTitleTemplate)}
      <section
        class="carousel"
        role="region"
        aria-labelledby=${c(this.ariaLabelledByElement)}
        aria-label=${c(this.ariaLabelText)}
        aria-live="polite"
      >
        ${s(this.showNavBtns,()=>this.navLeftButtonTemplate)}
        ${this.initialSlotTemplate}
        <ul class="carousel-content" @scroll=${this.setNavVisibility}>
          ${this.carouselSlotsTemplate}
        </ul>
        ${s(this.showNavBtns,()=>this.navRightButtonTemplate)}
      </section>
    `}updated(e){if(e.has(`resizeObserver`)){let t=e.get(`resizeObserver`);t&&this.disconnectResizeObserver(t),this.setupResizeObserver()}}get carouselTitleTemplate(){return r`
      <div id="carousel-title" class="carousel-title">
        ${this.carouselTitle}
      </div>
    `}get ariaLabelText(){if(this.ariaApproach===`aria-label`)return this.customAriaLabel??i(`Carousel`)}get ariaLabelledByElement(){if(this.ariaApproach===`aria-labelledby`)return`carousel-title`}get ariaApproach(){return this.carouselTitle&&!this.customAriaLabel?`aria-labelledby`:`aria-label`}get initialSlotTemplate(){return r`<slot @slotchange=${this.handleSlotChange}></slot>`}get carouselSlotsTemplate(){return r`${Array.from({length:this.numCarouselItems},(e,t)=>r`
        <li class="carousel-slide">
          <slot name="slide-${t}"></slot>
        </li>
      `)}`}get navLeftButtonTemplate(){return r`
      <button
        class="carousel__btn ${this.navPreviousVisible?``:`disabled`}"
        title=${i(`View previous items`)}
        aria-label=${i(`Previous`)}
        @click=${this.onNavPreviousClick}
      >
        &#5176;
      </button>
    `}get navRightButtonTemplate(){return r`
      <button
        class="carousel__btn ${this.navNextVisible?``:`disabled`}"
        title=${i(`View next items`)}
        aria-label=${i(`Next`)}
        @click=${this.onNavNextClick}
      >
        &#5171;
      </button>
    `}handleSlotChange(e){let t=e.target.assignedElements();t.length&&(this.numCarouselItems=t.length),t.forEach((e,t)=>e.slot=`slide-${t}`),this.setNavVisibility()}setupResizeObserver(){this.resizeObserver&&this.carouselContent&&this.resizeObserver.addObserver({target:this.carouselContent,handler:this})}disconnectResizeObserver(e){this.carouselContent&&e.removeObserver({target:this.carouselContent,handler:this})}handleResize(e){e.target===this.carouselContent&&this.setNavVisibility()}onNavPreviousClick(e){this.carouselContent.scrollBy({top:0,left:-this.carouselContent.clientWidth,behavior:`smooth`}),e.preventDefault()}onNavNextClick(e){this.carouselContent.scrollBy({top:0,left:this.carouselContent.clientWidth,behavior:`smooth`}),e.preventDefault()}setNavVisibility(){if(this.showNavBtns=!0,this.carouselContent){let e=this.carouselContent.scrollLeft;this.navPreviousVisible=e>0;let t=this.carouselContent.scrollWidth-this.carouselContent.clientWidth-1;this.navNextVisible=e<t,!this.navPreviousVisible&&!this.navNextVisible&&(this.showNavBtns=!1)}}static get styles(){return e`
      .carousel-content {
        width: var(--basic-carousel-content-width, auto);
        margin: 0;
        padding: 0;
        list-style-type: none;
        scrollbar-width: none;
        box-sizing: border-box;
        overflow-x: scroll;
        scroll-snap-type: x mandatory;
        scrollbar-width: none;
        display: flex;
        flex-direction: row;
        gap: var(--basic-carousel-tile-gap, 20px);
      }

      .carousel-slide {
        scroll-snap-align: start;
        min-width: max-content;
      }

      .carousel {
        display: flex;
        flex-direction: row;
        gap: 10px;
      }

      .carousel-title {
        font-weight: 200;
        font-size: 2.4rem;
        margin-top: 2rem;
        margin-bottom: 1rem;
        text-align: var(--basic-carousel-title-text-align, auto);
      }

      .carousel__btn {
        color: var(--buttonFontColor, 'inherit');
        background-color: var(--buttonBGColor, 'transparent');
        display: flex;
        justify-content: center;
        align-items: center;
        border: none;
        cursor: pointer;
        min-width: 30px;
        min-height: 30px;
        font-size: 1.3rem;
        font-family: sans-serif;
        text-decoration: none;
        transition: opacity 0.2s;
      }

      .carousel__btn.disabled {
        opacity: 0.2;
        cursor: auto;
        pointer-events: none;
        visibility: visible;
      }
    `}};d([t({type:Object})],f.prototype,`resizeObserver`,void 0),d([t({type:String})],f.prototype,`carouselTitle`,void 0),d([t({type:String})],f.prototype,`customAriaLabel`,void 0),d([t({type:Boolean})],f.prototype,`titleSrOnly`,void 0),d([a()],f.prototype,`numCarouselItems`,void 0),d([a()],f.prototype,`navPreviousVisible`,void 0),d([a()],f.prototype,`navNextVisible`,void 0),d([a()],f.prototype,`showNavBtns`,void 0),d([u(`.carousel-content`)],f.prototype,`carouselContent`,void 0),f=d([l(`basic-carousel`),n()],f);