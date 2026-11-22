import{A as e,C as t,E as n,F as r,N as i,O as a,P as o,S as s,T as c,m as l,w as u,x as d}from"./vendor-lit-MQ4DzmMc.js";import{t as f}from"./index-NFRVgqV2.js";import{t as p}from"./plural-count-CuSJx8jp.js";import{t as m}from"./format-show-date-DvNvho4x.js";import{c as h,h as g,l as _,n as v,t as y}from"./models-BSAzn-k3.js";import{r as b}from"./show-urls-vhuWq7P2.js";import"./z-tokens-C3UDJHq4.js";var x=`https://archive.org`;function S(e,t=x){return`${t}/services/img/${e}`}var C=class extends c{render(){return a``}static get styles(){return e`
      /*
       * Two background layers. The base grey gradient mirrors the site's
       * collection-browser loading tiles (iaux-collection-browser
       * src/tiles/collection-browser-loading-tile.ts); the greys are
       * hardcoded there and no ia-theme token maps to them, so they are
       * hardcoded here for site consistency. Above it sits the shimmer: a
       * slightly diagonal white highlight band on an image three times the
       * host's width, so the animation can park the band offscreen at
       * either end and sweep it across by sliding background-position.
       */
      :host {
        display: block;
        width: 100%;
        height: 100%;
        border-radius: var(--tileCornerRadius, 4px);
        background-image:
          linear-gradient(
            110deg,
            transparent 42%,
            rgba(255, 255, 255, 0.1) 50%,
            transparent 58%
          ),
          linear-gradient(to right bottom, #dddddd, #d8d8d8);
        background-size:
          300% 100%,
          100% 100%;
        background-position:
          0% 0,
          0 0;
        background-repeat: no-repeat;
        animation: tombstone-shimmer 3s ease-in-out infinite;
      }

      /*
       * One sweep per three-second cycle: the highlight crosses the host
       * during the first 30% of the cycle and rests offscreen for the
       * remainder. The end-of-cycle jump back to the start position is
       * invisible because the band is parked outside the visible third of
       * the background image at both ends.
       */
      @keyframes tombstone-shimmer {
        0% {
          background-position:
            0% 0,
            0 0;
        }
        30%,
        100% {
          background-position:
            100% 0,
            0 0;
        }
      }

      @media (prefers-reduced-motion: reduce) {
        :host {
          animation: none;
        }
      }
    `}};C=f([u(`lma-tombstone`)],C);var w=class extends c{constructor(...e){super(...e),this.mode=`desktop`,this.previewable=!1,this.countsPending=!1,this.compact=!1,this.previewRequested=!1,this.previewShown=!1,this.pointerHold=!1,this.focusHold=!1,this.restoringFocus=!1,this.lastPointerDownTime=-1/0}restorePreviewFocus(){this.restoringFocus=!0;try{this.nameButton?.focus()}finally{this.restoringFocus=!1}}disconnectedCallback(){super.disconnectedCallback(),this.pointerHold=!1,this.focusHold=!1,this.resetPreview(this.artist)}willUpdate(e){if(!e.has(`artist`))return;let t=e.get(`artist`);t?.identifier!==this.artist?.identifier&&this.resetPreview(t)}render(){let e=this.artist;return e?a`
      <div
        class="tile"
        @pointerenter=${this.handlePointerEnter}
        @pointerleave=${this.handlePointerLeave}
        @pointerdown=${this.handlePointerDown}
        @focusin=${this.handleFocusIn}
        @focusout=${this.handleFocusOut}
      >
        <div class="art">
          <img src=${e.imageUrl} alt="" loading="lazy" decoding="async" />
        </div>
        <button class="artist-name" @click=${this.handleNameClick}>
          <span class="artist-name-text">${e.name}</span>
        </button>
        ${this.showCountTemplate(e)}
      </div>
    `:a``}showCountTemplate(e){let t=e.showCount;return t===null?this.countsPending?a`
        <div class="show-count" aria-hidden="true">
          <lma-tombstone class="count-tombstone"></lma-tombstone>
        </div>
      `:a`<div class="show-count"></div>`:a`<div class="show-count">${p(t,{one:()=>o(`1 show`),other:e=>o(r`${e} shows`)})}</div>`}handleNameClick(){let e=this.artist;e&&this.dispatchEvent(y(_.artistSelected,{artist:e}))}get hoverEnabled(){return this.previewable&&this.mode===`desktop`}handlePointerEnter(e){e.pointerType!==`mouse`||!this.hoverEnabled||(this.pointerHold=!0,this.engagePreview(`pointer`))}handlePointerLeave(){this.pointerHold=!1,this.releasePreview()}handlePointerDown(e){this.lastPointerDownTime=e.timeStamp}handleFocusIn(e){if(this.restoringFocus||this.focusHold||!this.hoverEnabled||e.timeStamp-this.lastPointerDownTime<100)return;let t=this.nameButton;!t||e.composedPath()[0]!==t||(this.focusHold=!0,this.engagePreview(`focus`))}handleFocusOut(e){let t=e.relatedTarget;t instanceof Node&&this.shadowRoot?.contains(t)||(this.focusHold=!1,this.releasePreview())}engagePreview(e){this.cancelPreviewTimer();let t=this.artist;t&&this.previewShown&&!this.previewRequested&&(this.previewRequested=!0,this.dispatchEvent(y(_.artistHovered,{artist:t,anchorRect:this.getBoundingClientRect(),via:e,opener:this,resumed:!0}))),this.previewTimer=setTimeout(()=>{this.previewTimer=void 0,this.requestPreview(e)},350)}releasePreview(){if(this.pointerHold||this.focusHold||(this.cancelPreviewTimer(),!this.previewRequested))return;this.previewRequested=!1;let e=this.artist;e&&this.dispatchEvent(y(_.artistHoverEnded,{artist:e}))}requestPreview(e){let t=this.artist;!t||!this.hoverEnabled||(this.previewRequested=!0,this.previewShown=!0,this.dispatchEvent(y(_.artistHovered,{artist:t,anchorRect:this.getBoundingClientRect(),via:e,opener:this})))}resetPreview(e){this.cancelPreviewTimer();let t=this.previewRequested;this.previewRequested=!1,this.previewShown=!1,!(!t||!e)&&this.dispatchEvent(y(_.artistHoverEnded,{artist:e}))}cancelPreviewTimer(){this.previewTimer!==void 0&&(clearTimeout(this.previewTimer),this.previewTimer=void 0)}static get styles(){return e`
      :host {
        display: block;
      }

      .tile {
        position: relative;
        cursor: pointer;
        text-align: center;
      }

      .art {
        position: relative;
        aspect-ratio: 1 / 1;
        overflow: hidden;
        margin: 8px 0 4px;
        border-radius: 50%;
        box-shadow: 0 0 0 2px var(--ia-theme-primary-cta-border, #c5d1df);
        transition: box-shadow 0.15s cubic-bezier(0.4, 0, 0.2, 1);
      }

      .tile:hover .art {
        box-shadow: 0 0 0 2px var(--ia-theme-link-color, #4b64ff);
      }

      .art img {
        display: block;
        width: 100%;
        height: 100%;
        object-fit: cover;
        transition: scale 0.15s cubic-bezier(0.4, 0, 0.2, 1);
      }

      .tile:hover .art img {
        scale: 1.05;
      }

      /* The name is a text-styled native button: it looks like the plain
         name text but carries the tile's accessible name and keyboard
         activation. */
      .artist-name {
        display: block;
        width: 100%;
        margin: 0 0 4px;
        padding: 0;
        border: 0;
        background: none;
        color: inherit;
        font-family: inherit;
        font-size: 14px;
        line-height: 20px;
        font-weight: 500;
        text-align: center;
        cursor: pointer;
      }

      /* A long artist name truncates to a single ellipsized line. */
      .artist-name-text {
        display: block;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
      }

      /* The stretched overlay makes the whole tile the name button's hit
         area. */
      .artist-name::after {
        content: '';
        position: absolute;
        inset: 0;
      }

      .artist-name:focus-visible {
        outline: none;
      }

      .artist-name:focus-visible::after {
        outline: 2px solid var(--ia-theme-link-color, #4b64ff);
        outline-offset: 2px;
        border-radius: 8px;
      }

      /* The count line's height stays reserved while the count is
         unresolved, so its text arriving does not shift the tile. */
      .show-count {
        height: 16px;
        font-size: 12px;
        line-height: 16px;
        color: var(--ia-theme-secondary-text-color, #666);
      }

      /* While the count resolves, the line holds a shimmering bar roughly
         the size of the count text it stands in for, centered in the
         line's fixed height. */
      .count-tombstone {
        width: 60px;
        height: 12px;
        margin: 2px auto;
      }

      /* Compact cell: the art is a fixed circle centered in the cell, the
         name wraps to at most two lines inside a fixed two-line height, and
         the count line is smaller. */
      :host([compact]) .art {
        width: 96px;
        height: 96px;
        margin: 0 auto 8px;
      }

      :host([compact]) .artist-name {
        height: 32px;
        font-size: 12px;
        line-height: 16px;
      }

      :host([compact]) .artist-name-text {
        display: -webkit-box;
        -webkit-box-orient: vertical;
        -webkit-line-clamp: 2;
        white-space: normal;
        overflow-wrap: anywhere;
      }

      :host([compact]) .show-count {
        height: 14px;
        font-size: 10px;
        line-height: 14px;
      }

      :host([compact]) .count-tombstone {
        width: 40px;
        height: 10px;
      }
    `}};f([t({type:Object})],w.prototype,`artist`,void 0),f([t({type:String})],w.prototype,`mode`,void 0),f([t({type:Boolean})],w.prototype,`previewable`,void 0),f([t({type:Boolean})],w.prototype,`countsPending`,void 0),f([t({type:Boolean,reflect:!0})],w.prototype,`compact`,void 0),f([d(`button.artist-name`)],w.prototype,`nameButton`,void 0),w=f([u(`lma-artist-tile`),i()],w);function T(e){return e.split(``).reduce((e,t)=>e+t.charCodeAt(0),0)}function E(e){return`waveform-grad${T(e)%6}`}var D=new Set([h,`stream_only`]);function O(e){return e.collections.find(e=>!!e&&!D.has(e)&&!e.startsWith(`fav-`))}function k(e){return O(e)||e.artist.replace(/[^A-Za-z0-9]/g,``)}function A(e){let t=k(e);if(!t)return;let n=/^(https?:\/\/[^/]+)\/services\/img\//.exec(e.imageUrl)?.[1];return S(t,n)}var j=e`
  .waveform-wash {
    position: absolute;
    inset: 0;
    background: linear-gradient(var(--waveformTop--), var(--waveformMid--));
    opacity: 0.22;
  }

  /* The 2px white cap is the band's own top row: when the hover zoom snaps
     the band's edge and the image's edge to different device pixels, the
     row the image leaves uncovered is white, which the multiply blend drops
     out, instead of the peak colour. */
  .waveform-band {
    position: absolute;
    left: 0;
    bottom: 1px;
    width: 100%;
    height: 22%;
    overflow: hidden;
    isolation: isolate;
    mix-blend-mode: multiply;
    background: linear-gradient(
      #fff 0,
      #fff 2px,
      var(--waveformPeak--) 2px,
      var(--waveformTop--) 55%,
      var(--waveformMid--) 80%,
      var(--waveformLow--)
    );
  }

  .waveform-band .waveform {
    position: absolute;
    left: 0;
    bottom: -150%;
    width: 100%;
    height: 300%;
    filter: invert(1);
    mix-blend-mode: screen;
  }

  .waveform-logo {
    position: absolute;
    left: 50%;
    top: 38%;
    width: 52%;
    height: 52%;
    translate: -50% -50%;
    object-fit: contain;
    pointer-events: none;
  }

  @media (forced-colors: active) {
    .waveform-wash {
      display: none;
    }

    .waveform-band {
      mix-blend-mode: normal;
      background: none;
    }

    .waveform-band .waveform {
      filter: none;
      mix-blend-mode: normal;
    }
  }

  .waveform-grad0 {
    --waveformPeak--: hsl(300, 80%, 64%);
    --waveformTop--: hsl(300, 80%, 55%);
    --waveformMid--: hsl(330, 80%, 33%);
    --waveformLow--: hsl(330, 80%, 22%);
  }

  .waveform-grad1 {
    --waveformPeak--: hsl(200, 80%, 64%);
    --waveformTop--: hsl(200, 80%, 55%);
    --waveformMid--: hsl(230, 80%, 33%);
    --waveformLow--: hsl(230, 80%, 22%);
  }

  .waveform-grad2 {
    --waveformPeak--: hsl(160, 80%, 64%);
    --waveformTop--: hsl(160, 80%, 55%);
    --waveformMid--: hsl(190, 80%, 33%);
    --waveformLow--: hsl(190, 80%, 22%);
  }

  .waveform-grad3 {
    --waveformPeak--: hsl(250, 80%, 64%);
    --waveformTop--: hsl(250, 80%, 55%);
    --waveformMid--: hsl(280, 80%, 33%);
    --waveformLow--: hsl(280, 80%, 22%);
  }

  .waveform-grad4 {
    --waveformPeak--: hsl(280, 80%, 64%);
    --waveformTop--: hsl(280, 80%, 55%);
    --waveformMid--: hsl(310, 80%, 33%);
    --waveformLow--: hsl(310, 80%, 22%);
  }

  .waveform-grad5 {
    --waveformPeak--: hsl(340, 80%, 64%);
    --waveformTop--: hsl(340, 80%, 55%);
    --waveformMid--: hsl(0, 80%, 33%);
    --waveformLow--: hsl(0, 80%, 22%);
  }
`;function M(e){return e.naturalWidth/e.naturalHeight===4}function N(e){return e.naturalWidth===160&&e.naturalHeight===110}var P=new Map;function F(e){P.set(e,!0)}function I(e){return P.get(e)===!0}var L=new Set;function R(e){L.add(e)}function z(e){return L.has(e)}var B=class extends c{constructor(...e){super(...e),this.mode=`desktop`,this.capabilities=v,this.previewable=!1,this.waveformArt=!1,this.artLoaded=!1,this.logoFailed=!1,this.previewRequested=!1,this.previewShown=!1,this.pointerHold=!1,this.focusHold=!1,this.restoringFocus=!1,this.buttonPreviewRequested=!1,this.lastPointerDownTime=-1/0,this.previewVia=`pointer`}restorePreviewFocus(){let e=(this.previewVia===`button`?this.previewButton:void 0)??this.titleButton;this.restoringFocus=!0;try{e?.focus()}finally{this.restoringFocus=!1}}disconnectedCallback(){super.disconnectedCallback(),this.pointerHold=!1,this.focusHold=!1,this.resetPreview(this.show)}willUpdate(e){if(e.has(`show`)){let t=e.get(`show`);t?.identifier!==this.show?.identifier&&this.resetPreview(t);let n=this.show?.identifier;this.waveformArt=n?I(n):!1,this.logoFailed=!1,t?.imageUrl!==this.show?.imageUrl&&(this.artLoaded=!1)}}render(){let e=this.show;if(!e)return a``;let t=e.date.split(`T`)[0];return a`
      <div
        class="tile ${this.mode}"
        @pointerenter=${this.handlePointerEnter}
        @pointerleave=${this.handlePointerLeave}
        @pointerdown=${this.handlePointerDown}
        @focusin=${this.handleFocusIn}
        @focusout=${this.handleFocusOut}
      >
        <button class="show-title" @click=${this.handleTitleClick}>
          <span class="show-title-text">${e.title}</span>
        </button>
        ${this.artTemplate(e)}
        <div class="show-venue">${e.venue??n}</div>
        <time class="show-date" datetime=${t}>
          ${m(e.date)}
        </time>
        <div class="rating-row">
          ${this.ratingTemplate(e)} ${this.reviewCountTemplate(e)}
        </div>
        ${this.previewButtonTemplate(e)}
      </div>
    `}previewButtonTemplate(e){let t=this.mode!==`desktop`;if(!this.previewable||!t)return n;let i=o(r`Show details for ${e.title}`);return a`
      <button
        class="preview-button"
        title=${i}
        aria-label=${i}
        @click=${this.handlePreviewButtonClick}
      >
        ${g(`info`,{size:12})}
      </button>
    `}artTemplate(e){let t=this.waveformArt,r=l({art:!0,"waveform-art":t,[E(e.identifier)]:t}),i=l({"art-frame":!0,"waveform-band":t,revealed:this.artLoaded}),o=l({waveform:t,cover:!t,revealed:this.artLoaded});return a`
      <div class=${r}>
        <div class="face">
          ${t?a`<div class="waveform-wash"></div>`:n}
          <div class=${i}>
            <img
              class=${o}
              src=${e.imageUrl}
              alt=""
              loading="lazy"
              decoding="async"
              @load=${this.handleArtLoad}
            />
          </div>
          ${t?this.logoTemplate(e):n}
        </div>
        ${this.queueButtonTemplate} ${this.downloadLinkTemplate(e)}
        ${this.playButtonTemplate}
      </div>
    `}logoTemplate(e){let t=A(e);return!t||this.logoFailed||z(t)?n:a`
      <img
        class="waveform-logo"
        src=${t}
        alt=""
        loading="lazy"
        decoding="async"
        @load=${this.handleLogoLoad}
        @error=${this.handleLogoError}
      />
    `}get queueButtonTemplate(){if(!this.capabilities.playbackAvailable)return n;let e=o(`Add to queue`);return a`
      <button
        class="tile-action queue"
        title=${e}
        aria-label=${e}
        @click=${this.handleQueueClick}
      >
        ${g(`plus`)}
      </button>
    `}downloadLinkTemplate(e){if(e.streamOnly)return n;let t=o(`Download this show (archive.org)`);return a`
      <a
        class="tile-action download"
        href=${b(e.identifier)}
        target="_blank"
        rel="noopener"
        title=${t}
        aria-label=${t}
      >
        ${g(`download`)}
      </a>
    `}get playButtonTemplate(){if(!this.capabilities.playbackAvailable)return n;let e=o(`Play show`);return a`
      <button
        class="tile-action play"
        title=${e}
        aria-label=${e}
        @click=${this.handlePlayClick}
      >
        ${g(`play`)}
      </button>
    `}ratingTemplate(e){let t=e.avgRating;return t===null?a`<span class="not-rated">${o(`Not yet rated`)}</span>`:a`
      <span class="rating">
        ${g(`star`,{size:12})}
        <span>${o(r`${t.toFixed(1)}/5`)}</span>
      </span>
    `}reviewCountTemplate(e){let t=e.numReviews;return t<1?n:a`<span class="review-count">${p(t,{one:()=>o(`1 review`),other:e=>o(r`${e} reviews`)})}</span>`}handleArtLoad(e){let t=e.target;if(!(t instanceof HTMLImageElement))return;let n=M(t),r=this.show;r&&n&&F(r.identifier),this.waveformArt=n,this.artLoaded=!0}handleLogoLoad(e){let t=e.target;if(!(t instanceof HTMLImageElement)||!N(t))return;let n=t.getAttribute(`src`);n&&(R(n),this.requestUpdate())}handleLogoError(){this.logoFailed=!0}handlePointerEnter(e){e.pointerType!==`mouse`||this.mode!==`desktop`||(this.pointerHold=!0,this.engagePreview(`pointer`))}handlePointerLeave(){this.pointerHold=!1,this.releasePreview()}handlePointerDown(e){this.lastPointerDownTime=e.timeStamp}handleFocusIn(e){if(this.restoringFocus||this.focusHold||this.mode!==`desktop`||e.timeStamp-this.lastPointerDownTime<100)return;let t=this.titleButton;!t||e.composedPath()[0]!==t||(this.focusHold=!0,this.engagePreview(`focus`))}handleFocusOut(e){let t=e.relatedTarget;t instanceof Node&&this.shadowRoot?.contains(t)||(this.focusHold=!1,this.releasePreview())}engagePreview(e){this.cancelPreviewTimer();let t=this.show;t&&this.previewShown&&!this.previewRequested&&(this.previewRequested=!0,this.dispatchEvent(y(_.showPreviewResumed,{show:t}))),this.previewTimer=setTimeout(()=>{this.previewTimer=void 0,this.requestPreview(e)},350)}releasePreview(){if(this.pointerHold||this.focusHold||(this.cancelPreviewTimer(),!this.previewRequested))return;this.previewRequested=!1;let e=this.show;e&&this.dispatchEvent(y(_.showPreviewEnded,{show:e}))}requestPreview(e){let t=this.show;!t||this.mode!==`desktop`||(this.previewRequested=!0,this.previewShown=!0,this.previewVia=e,this.dispatchEvent(y(_.showPreviewRequested,{show:t,anchorRect:this.getBoundingClientRect(),via:e,opener:this})))}handlePreviewButtonClick(){let e=this.show;e&&(this.previewVia=`button`,this.buttonPreviewRequested=!0,this.dispatchEvent(y(_.showPreviewRequested,{show:e,anchorRect:this.getBoundingClientRect(),via:`button`,opener:this})))}resetPreview(e){this.cancelPreviewTimer();let t=this.previewRequested||this.buttonPreviewRequested;this.previewRequested=!1,this.buttonPreviewRequested=!1,this.previewShown=!1,!(!t||!e)&&this.dispatchEvent(y(_.showPreviewEnded,{show:e}))}cancelPreviewTimer(){this.previewTimer!==void 0&&(clearTimeout(this.previewTimer),this.previewTimer=void 0)}handleTitleClick(){this.emitShowSelected()}handlePlayClick(){let e=this.show;e&&this.dispatchEvent(y(_.showPlayRequested,{show:e}))}handleQueueClick(){let e=this.show;e&&this.dispatchEvent(y(_.showQueueRequested,{show:e}))}emitShowSelected(){let e=this.show;e&&this.dispatchEvent(y(_.showSelected,{show:e}))}static get styles(){return[j,e`
        :host {
          display: block;
        }

        /* The tile fills its rail slot's stretched height so every tile in
           a rail row bottoms out at the same edge; the rating row's auto
           top margin absorbs the slack. */
        .tile {
          position: relative;
          display: flex;
          flex-direction: column;
          height: 100%;
          cursor: pointer;
        }

        /* The art sits after the title button in the DOM so keyboard focus
           reaches the whole-tile title activator before the action trio;
           order pulls the art back to the top visually without affecting
           the tab sequence. */
        .art {
          order: -1;
          position: relative;
          aspect-ratio: 1 / 1;
          overflow: hidden;
          margin-bottom: 12px;
          border-radius: 8px;
          /* The plain grey is the placeholder surface behind the image
             while it is still invisible awaiting classification, matching
             the tombstones' collection-browser grey, which no
             ia-theme token maps to. It is set as background-color, the
             same property the waveform rule sets, so that rule's
             near-white ground overrides it. */
          background-color: #dddddd;
          box-shadow: 0 0 0 2px var(--ia-theme-primary-cta-border, #c5d1df);
          transition: box-shadow 0.15s cubic-bezier(0.4, 0, 0.2, 1);
        }

        .tile:hover .art {
          box-shadow: 0 0 0 2px var(--ia-theme-link-color, #4b64ff);
        }

        /* The waveform tile's near-white ground is part of the tile's
           artwork, like cover art, so it stays light in every theme and no
           theme token describes it. The face paints it so that it lies
           inside the face's isolated blend group, where the band's multiply
           blend can see it: an isolated group starts out transparent, and a
           ground painted only on the art would leave the band compositing
           its peak-free areas as plain white, a shade lighter than the
           ground around it. The art paints it too, so the anti-aliased
           edge of the rounded corners shows no placeholder grey. */
        .art.waveform-art,
        .art.waveform-art .face {
          background-color: #fafaf9;
        }

        /* The face carries every layer the hover zoom scales. It isolates
           its own blend group, so the waveform band's multiply blend
           composites against the washed ground only, whether or not the
           zoom is active. */
        .face {
          position: absolute;
          inset: 0;
          isolation: isolate;
          transition: scale 0.15s cubic-bezier(0.4, 0, 0.2, 1);
        }

        .tile:hover .face {
          scale: 1.05;
        }

        /* Outside waveform mode the frame fills the face with the cover
           art; in waveform mode it is the bottom waveform band. */
        .art-frame:not(.waveform-band) {
          position: absolute;
          inset: 0;
        }

        .art-frame img {
          display: block;
          width: 100%;
          opacity: 0;
        }

        /* The image is invisible until its load handler classifies it and
           reveals it in the same update, so its first visible paint is
           already in the correct mode; a progressively-painting stock 4:1
           waveform in cover mode would otherwise show as a full-crop
           square while streaming. The waveform band hides with it, so its
           gradient background never shows as a solid bar. */
        .art-frame img.revealed {
          opacity: 1;
        }

        .waveform-band:not(.revealed) {
          opacity: 0;
        }

        .art-frame img.cover {
          height: 100%;
          object-fit: cover;
        }

        /* The action circles are deliberately translucent white with dark
           icons in every theme: they sit over arbitrary artwork, which no
           theme token describes. The tile-action z token layers them above
           the title button's whole-tile overlay so they receive their own
           clicks and focus. */
        .tile-action {
          position: absolute;
          z-index: ${2};
          bottom: 8px;
          width: 32px;
          height: 32px;
          display: flex;
          align-items: center;
          justify-content: center;
          box-sizing: border-box;
          border: 0;
          padding: 0;
          border-radius: 9999px;
          background: rgba(255, 255, 255, 0.6);
          color: #171717;
          text-decoration: none;
          box-shadow:
            0 4px 6px -1px rgb(0 0 0 / 0.1),
            0 2px 4px -2px rgb(0 0 0 / 0.1);
          cursor: pointer;
          transition:
            opacity 0.15s cubic-bezier(0.4, 0, 0.2, 1),
            background-color 0.15s cubic-bezier(0.4, 0, 0.2, 1),
            backdrop-filter 0s linear 0.15s;
        }

        .tile-action:hover {
          background: rgba(255, 255, 255, 0.95);
        }

        .tile-action.queue {
          left: 8px;
        }

        .tile-action.download {
          left: 50%;
          transform: translateX(-50%);
        }

        .tile-action.play {
          right: 8px;
        }

        .tile-action.play svg {
          fill: currentColor;
        }

        .tile.desktop .tile-action {
          opacity: 0;
        }

        /* In desktop mode the circles frost the artwork behind them while
           the tile is hovered or holds focus: the frost comes on at once
           and goes off after the fade-out. Idle desktop tiles, and every
           tile outside desktop mode, carry no backdrop filter. */
        .tile.desktop:hover .tile-action,
        .tile.desktop:focus-within .tile-action {
          opacity: 1;
          backdrop-filter: blur(2px);
          transition:
            opacity 0.15s cubic-bezier(0.4, 0, 0.2, 1),
            background-color 0.15s cubic-bezier(0.4, 0, 0.2, 1),
            backdrop-filter 0s;
        }

        /* The title is a text-styled native button: it looks like the plain
           title text but carries the tile's accessible name and keyboard
           activation. */
        .show-title {
          display: block;
          width: 100%;
          margin: 0 0 4px;
          padding: 0;
          border: 0;
          background: none;
          color: inherit;
          font-family: inherit;
          font-size: 14px;
          line-height: 20px;
          font-weight: 700;
          text-align: left;
          cursor: pointer;
        }

        .show-title-text {
          display: -webkit-box;
          -webkit-box-orient: vertical;
          -webkit-line-clamp: 2;
          overflow: hidden;
        }

        /* The stretched overlay makes the whole tile the title button's hit
           area. The tile-overlay z token lifts it above the positioned art
           block, which follows it in the DOM and would otherwise paint over
           it and take its clicks. */
        .show-title::after {
          content: '';
          position: absolute;
          inset: 0;
          z-index: ${1};
        }

        .show-title:focus-visible {
          outline: none;
        }

        .show-title:focus-visible::after {
          outline: 2px solid var(--ia-theme-link-color, #4b64ff);
          outline-offset: 2px;
          border-radius: 8px;
        }

        .show-venue,
        .show-date {
          font-size: 12px;
          line-height: 16px;
          font-weight: 700;
          color: var(--ia-theme-secondary-text-color, #666);
        }

        /* The venue line holds its one-line height even when the show has
           no venue, and a long venue name truncates to a single ellipsized
           line, so the tile keeps the height the loading placeholder
           reserves for it. */
        .show-venue {
          height: 16px;
          white-space: nowrap;
          overflow: hidden;
          text-overflow: ellipsis;
        }

        .show-date {
          display: block;
          margin-bottom: 8px;
        }

        /* The auto top margin pushes the rating row to the tile's bottom
           edge, so a tile stretched past its natural height (to match a
           taller rail neighbor) collects the slack above the row and every
           rating row in the rail bottom-aligns. */
        .rating-row {
          display: flex;
          align-items: center;
          justify-content: space-between;
          margin-top: auto;
          height: 16px;
          font-size: 12px;
          line-height: 16px;
          color: var(--ia-theme-secondary-text-color, #666);
        }

        .rating {
          display: flex;
          align-items: center;
          gap: 4px;
          min-width: 0;
        }

        /* Rating-star yellow: no theme token describes a rating accent. */
        .rating svg {
          flex-shrink: 0;
          color: #facc15;
          fill: #facc15;
        }

        .review-count {
          flex-shrink: 0;
        }

        /* The (i) preview button sits at the right end of the date line,
           above the whole-tile overlay so a tap on it opens the preview
           instead of selecting the show. Its pseudo-element widens the
           touch target past the 20px circle. */
        .preview-button {
          position: absolute;
          z-index: ${2};
          right: 0;
          bottom: 22px;
          width: 20px;
          height: 20px;
          display: flex;
          align-items: center;
          justify-content: center;
          box-sizing: border-box;
          padding: 0;
          border: 1px solid var(--ia-theme-secondary-text-color, #666);
          border-radius: 9999px;
          background: var(--ia-theme-secondary-background-color, #fff);
          color: var(--ia-theme-secondary-text-color, #666);
          cursor: pointer;
          transition:
            border-color 0.15s cubic-bezier(0.4, 0, 0.2, 1),
            color 0.15s cubic-bezier(0.4, 0, 0.2, 1);
        }

        .preview-button::before {
          content: '';
          position: absolute;
          inset: -8px;
        }

        .preview-button:hover {
          border-color: var(--ia-theme-link-color, #4b64ff);
          color: var(--ia-theme-link-color, #4b64ff);
        }
      `]}};f([t({type:Object})],B.prototype,`show`,void 0),f([t({type:String})],B.prototype,`mode`,void 0),f([t({type:Object})],B.prototype,`capabilities`,void 0),f([t({type:Boolean})],B.prototype,`previewable`,void 0),f([s()],B.prototype,`waveformArt`,void 0),f([s()],B.prototype,`artLoaded`,void 0),f([s()],B.prototype,`logoFailed`,void 0),f([d(`button.show-title`)],B.prototype,`titleButton`,void 0),f([d(`button.preview-button`)],B.prototype,`previewButton`,void 0),B=f([u(`lma-show-tile`),i()],B);export{O as n,S as r,A as t};